#![allow(clippy::single_match)]

use core::array::from_fn;

use embassy_time::Instant;
use usbd_hid::descriptor::{KeyboardReport, MediaKeyboardReport, MouseReport};

use crate::{
    constant::{AUTO_MOUSE_DURATION, AUTO_MOUSE_LAYER, COLS, LAYER_NUM, ROWS},
    driver::keyboard::{Hand, KeyChangeEventOneHand},
    keyboard::keycode::{KeyDef, Layer},
};

use self::{
    all_pressed::{AllPressed, KeyStatusChangeType},
    event::process_event,
    report_gen::{
        keyboard::KeyboardReportGenerator, media_keyboard::MediaKeyboardReportGenerator,
        mouse::MouseReportGenerator,
    },
};

use super::keyboard::keycode::KeyAction;

mod all_pressed;
mod event;
mod report_gen;

pub struct StateReport {
    pub keyboard_report: Option<KeyboardReport>,
    pub mouse_report: Option<MouseReport>,
    pub media_keyboard_report: Option<MediaKeyboardReport>,
    pub highest_layer: u8,
}

#[derive(Default)]
pub struct KeyState {
    force_hold: bool,
    layer: usize,
}

/// メインループ内部での状態
#[derive(Default)]
pub struct MainloopState {
    /// 自動マウスレイヤ即時無効化判定に使用
    normal_key_pressed: bool,
    keycodes: [u8; 6],
    keycodes_idx: usize,
    modifier: u8,
    media_key: Option<u16>,
    mouse_buttons: u8,
}

pub struct State {
    master_hand: Hand,

    layers: [Layer; LAYER_NUM],
    layer_active: [bool; LAYER_NUM],

    pressed: AllPressed,
    key_state: [[KeyState; COLS * 2]; ROWS],

    auto_mouse_start: Option<Instant>,
    scroll_mode: bool,

    kb_report_gen: KeyboardReportGenerator,
    mkb_report_gen: MediaKeyboardReportGenerator,
    mouse_report_gen: MouseReportGenerator,
}

impl State {
    pub fn new(layers: [Layer; LAYER_NUM], master_hand: Hand) -> Self {
        Self {
            master_hand,

            layers,
            layer_active: [false; LAYER_NUM],

            pressed: AllPressed::new(),
            key_state: from_fn(|_| from_fn(|_| KeyState::default())),

            auto_mouse_start: None,
            scroll_mode: false,

            kb_report_gen: KeyboardReportGenerator::new(),
            mkb_report_gen: MediaKeyboardReportGenerator::new(),
            mouse_report_gen: MouseReportGenerator::new(),
        }
    }

    pub fn update(
        &mut self,
        master_events: &mut [KeyChangeEventOneHand],
        slave_events: &mut [KeyChangeEventOneHand],
        mouse_event: &(i8, i8),
    ) -> StateReport {
        let now = Instant::now();
        let prev_highest_layer = self.highest_layer();
        let mut state = MainloopState::default();

        let events = {
            let (left_events, right_events) = if self.master_hand == Hand::Left {
                (master_events, slave_events)
            } else {
                (slave_events, master_events)
            };
            right_events.iter_mut().for_each(|event| {
                event.col = ((COLS - 1) as u8 - event.col) + COLS as u8;
            });
            let both_events = right_events.iter().chain(left_events.iter());

            self.pressed
                .compose_events_and_update_pressed(both_events, now)
        };

        for e in events.iter() {
            let Some(ka) = self.get_keycode(e.row, e.col, prev_highest_layer) else {
                continue;
            };
            process_event(
                &ka,
                e,
                &mut state,
                &mut self.layer_active,
                &mut self.scroll_mode,
            );

            if let KeyStatusChangeType::Released(_) = e.change_type {}
        }
        if state.normal_key_pressed {
            for e in events.iter() {}
        }

        if *mouse_event != (0, 0) || state.mouse_buttons != 0 || self.scroll_mode {
            self.layer_active[AUTO_MOUSE_LAYER] = true;
            self.auto_mouse_start = Some(now);
        } else if let Some(start) = self.auto_mouse_start {
            if now.duration_since(start) > AUTO_MOUSE_DURATION || state.normal_key_pressed {
                self.layer_active[AUTO_MOUSE_LAYER] = false;
                self.auto_mouse_start = None;
            }
        };

        StateReport {
            keyboard_report: self.kb_report_gen.gen(
                state.keycodes,
                state.modifier,
                state.keycodes_idx as u8,
            ),
            mouse_report: self.mouse_report_gen.gen(
                *mouse_event,
                state.mouse_buttons,
                self.scroll_mode,
            ),
            media_keyboard_report: self.mkb_report_gen.gen(state.media_key),
            highest_layer: prev_highest_layer as u8,
        }
    }

    fn highest_layer(&self) -> usize {
        self.layer_active.iter().rposition(|&x| x).unwrap_or(0)
    }

    fn get_keycode(&self, row: u8, col: u8, layer: usize) -> Option<KeyAction> {
        if row >= ROWS as u8 || col >= (COLS * 2) as u8 {
            return None;
        }

        for layer in (0..=layer).rev() {
            let key = &self.layers[layer][row as usize][col as usize];
            match key {
                KeyDef::None => return None,
                KeyDef::Inherit => continue,
                KeyDef::Key(action) => return Some(*action),
            }
        }

        None
    }
}
