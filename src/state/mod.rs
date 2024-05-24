#![allow(clippy::single_match)]

use core::array::from_fn;

use embassy_time::Instant;
use usbd_hid::descriptor::{KeyboardReport, MediaKeyboardReport, MouseReport};

use crate::{
    constant::{
        AUTO_MOUSE_LAYER, AUTO_MOUSE_TIME, COLS, LAYER_NUM, ROWS, SCROLL_DIVIDER_X,
        SCROLL_DIVIDER_Y, TAP_THRESHOLD,
    },
    driver::keyboard::{Hand, KeyChangeEventOneHand},
    keyboard::keycode::{layer::LayerOp, special::Special, KeyCode, KeyDef, Layer},
};

use self::all_pressed::{AllPressed, KeyStatusChangeType};

use super::keyboard::keycode::KeyAction;

mod all_pressed;

pub struct StateReport {
    pub keyboard_report: Option<KeyboardReport>,
    pub mouse_report: Option<MouseReport>,
    pub media_keyboard_report: Option<MediaKeyboardReport>,
    pub highest_layer: u8,
}

#[derive(Default)]
pub struct KeyState {
    normal_key_pressed: bool,
}

pub struct State {
    master_hand: Hand,

    layers: [Layer; LAYER_NUM],
    layer_active: [bool; LAYER_NUM],

    pressed: AllPressed,
    key_state: [[KeyState; COLS * 2]; ROWS],

    auto_mouse_start: Option<Instant>,
    // scoll_modeがonのときにSomeとなり、中身には「残っているスクロール」の値が入る。
    // スクロールは値が小さい関係上、1より小さい値になることが多々ある。これを0とみなすと、小さいスクロールができなくなってしまう。
    scroll_mode: Option<(i8, i8)>,

    empty_kb_sent: bool,
    empty_mouse_sent: bool,
    empty_mkb_sent: bool,
    previous_mkb: Option<MediaKeyboardReport>,
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
            scroll_mode: None,

            empty_kb_sent: false,
            empty_mouse_sent: false,
            empty_mkb_sent: false,
            previous_mkb: None,
        }
    }

    pub fn update(
        &mut self,
        master_events: &mut [KeyChangeEventOneHand],
        slave_events: &mut [KeyChangeEventOneHand],
        mouse_event: &(i8, i8),
    ) -> StateReport {
        let now = Instant::now();

        let (left_events, right_events) = if self.master_hand == Hand::Left {
            (master_events, slave_events)
        } else {
            (slave_events, master_events)
        };
        right_events.iter_mut().for_each(|event| {
            event.col = ((COLS - 1) as u8 - event.col) + COLS as u8;
        });

        let split_events = right_events.iter().chain(left_events.iter());

        let events = self.pressed.compose_events(split_events, now);
        let events = events
            .into_iter()
            .filter_map(|e| {
                let ka = self.get_keycode(e.row, e.col)?;
                Some((ka, e))
            })
            .collect::<heapless::Vec<_, 32>>();

        let mut keycodes = [0; 6];
        let mut keycodes_idx = 0;
        let mut modifier = 0;

        let mut mouse_buttons = 0;

        let mut media_key = None;

        // - 自動マウスレイヤ即時無効化判定
        // - TapHoldの強制Hold化判定
        // に使用
        let mut normal_key_pressed_enable = false;

        for (key_action, event) in events.iter() {
            let Some(kc) = (match event.change_type {
                KeyStatusChangeType::Pressed => match key_action {
                    KeyAction::Tap(kc) => Some(kc),
                    _ => None,
                },
                KeyStatusChangeType::Pressing(duration) => match key_action {
                    KeyAction::Tap(kc) => Some(kc),
                    KeyAction::TapHold(_tkc, hkc) => {
                        if duration.as_millis() > TAP_THRESHOLD {
                            Some(hkc)
                        } else {
                            None
                        }
                    }
                },
                KeyStatusChangeType::Released(duration) => match key_action {
                    KeyAction::Tap(kc) => Some(kc),
                    KeyAction::TapHold(tkc, hkc) => {
                        if duration.as_millis() > TAP_THRESHOLD {
                            Some(hkc)
                        } else {
                            Some(tkc)
                        }
                    }
                },
            }) else {
                continue;
            };

            match kc {
                KeyCode::Key(key) => {
                    if let KeyStatusChangeType::Pressed = event.change_type {
                        normal_key_pressed_enable = true;
                    }
                    if keycodes_idx < 6 {
                        keycodes[keycodes_idx] = *key as u8;
                        keycodes_idx += 1;
                    }
                }
                KeyCode::Media(key) => {
                    media_key = Some(*key as u16);
                }
                KeyCode::Mouse(btn) => mouse_buttons |= btn.bits(),
                KeyCode::Modifier(mod_key) => {
                    modifier |= mod_key.bits();
                }
                KeyCode::WithModifier(mod_key, key) => {
                    if keycodes_idx < 6 {
                        keycodes[keycodes_idx] = *key as u8;
                        modifier |= mod_key.bits();
                        keycodes_idx += 1;
                    }
                }
                KeyCode::Layer(layer_op) => match event.change_type {
                    KeyStatusChangeType::Released(_) => match layer_op {
                        LayerOp::Move(l) => {
                            self.layer_active[*l] = false;
                        }
                        LayerOp::Toggle(l) => {
                            self.layer_active[*l] = !self.layer_active[*l];
                        }
                    },
                    _ => match layer_op {
                        LayerOp::Move(l) => {
                            self.layer_active[*l] = true;
                        }
                        _ => {}
                    },
                },
                KeyCode::Special(special_op) => match event.change_type {
                    KeyStatusChangeType::Released(_) => match special_op {
                        Special::MoScrl => {
                            self.scroll_mode = None;
                        }
                    },
                    _ => match special_op {
                        Special::MoScrl => {
                            if self.scroll_mode.is_none() {
                                self.scroll_mode = Some((0, 0));
                            }
                        }
                    },
                },
            };
        }

        if *mouse_event != (0, 0) || mouse_buttons != 0 || self.scroll_mode.is_some() {
            self.layer_active[AUTO_MOUSE_LAYER] = true;
            self.auto_mouse_start = Some(now);
        } else if let Some(start) = self.auto_mouse_start {
            if now.duration_since(start).as_millis() > AUTO_MOUSE_TIME {
                self.layer_active[AUTO_MOUSE_LAYER] = false;
            }
        };

        let mouse_report = if *mouse_event == (0, 0) && mouse_buttons == 0 {
            if !self.empty_mouse_sent {
                self.empty_mouse_sent = true;
                Some(MouseReport {
                    x: 0,
                    y: 0,
                    buttons: 0,
                    wheel: 0,
                    pan: 0,
                })
            } else {
                None
            }
        } else {
            self.empty_mouse_sent = false;
            if let Some((remained_wheel, remained_pan)) = &mut self.scroll_mode {
                let wheel_raw = mouse_event.0 + *remained_wheel;
                let pan_raw = mouse_event.1 + *remained_pan;
                let wheel = wheel_raw / SCROLL_DIVIDER_Y;
                let pan = pan_raw / SCROLL_DIVIDER_X;
                *remained_wheel = wheel_raw % SCROLL_DIVIDER_Y;
                *remained_pan = pan_raw % SCROLL_DIVIDER_X;
                Some(MouseReport {
                    x: 0,
                    y: 0,
                    buttons: mouse_buttons,
                    wheel,
                    pan,
                })
            } else {
                Some(MouseReport {
                    x: mouse_event.1,
                    y: mouse_event.0,
                    buttons: mouse_buttons,
                    wheel: 0,
                    pan: 0,
                })
            }
        };

        let keyboard_report = if modifier == 0 && keycodes_idx == 0 {
            if !self.empty_kb_sent {
                self.empty_kb_sent = true;
                Some(KeyboardReport::default())
            } else {
                None
            }
        } else {
            self.empty_kb_sent = false;
            Some(KeyboardReport {
                keycodes,
                modifier,
                reserved: 0,
                leds: 0,
            })
        };

        let media_keyboard_report = match media_key {
            Some(key) => {
                self.empty_mkb_sent = false;
                let mut same = false;
                if let Some(prev) = &self.previous_mkb {
                    if prev.usage_id == key {
                        same = true;
                    }
                }
                if same {
                    None
                } else {
                    self.previous_mkb = Some(MediaKeyboardReport { usage_id: key });
                    Some(MediaKeyboardReport { usage_id: key })
                }
            }
            None => {
                self.previous_mkb = None;
                if !self.empty_mkb_sent {
                    self.empty_mkb_sent = true;
                    Some(MediaKeyboardReport { usage_id: 0 })
                } else {
                    None
                }
            }
        };

        StateReport {
            keyboard_report,
            mouse_report,
            media_keyboard_report,
            highest_layer: self.layer_active.iter().rposition(|&x| x).unwrap_or(0) as u8,
        }
    }

    fn get_keycode(&self, row: u8, col: u8) -> Option<KeyAction> {
        if row >= ROWS as u8 || col >= (COLS * 2) as u8 {
            return None;
        }

        let highest_layer = self.layer_active.iter().rposition(|&x| x).unwrap_or(0);

        for layer in (0..=highest_layer).rev() {
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
