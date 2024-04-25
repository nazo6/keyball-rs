#![allow(clippy::single_match)]

use core::array::from_fn;

use embassy_time::Instant;
use usbd_hid::descriptor::{KeyboardReport, MouseReport};

use crate::{
    constant::{AUTO_MOUSE_LAYER, AUTO_MOUSE_TIME, COLS, LAYER_NUM, ROWS, TAP_THRESHOLD},
    driver::keyboard::{Hand, KeyChangeEventOneHand},
    keyboard::keycode::{special::Special, KeyCode, Layer},
};

use self::{
    all_pressed::{AllPressed, KeyStatusChangeType},
    report::StateReport,
};

use super::keyboard::keycode::KeyAction;

mod all_pressed;
pub mod report;

#[derive(Default)]
pub struct KeyState {
    tapped: bool,
}

pub struct State {
    master_hand: Hand,

    layers: [Layer; LAYER_NUM],
    layer_active: [bool; LAYER_NUM],

    pressed: AllPressed,
    key_state: [[KeyState; COLS * 2]; ROWS],

    auto_mouse_start: Option<Instant>,
    scroll_mode: bool,

    empty_kb_sent: bool,
    empty_mouse_sent: bool,
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

            empty_kb_sent: false,
            empty_mouse_sent: false,
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

        let mut keycodes = [0; 6];
        let mut keycodes_idx = 0;

        let mut modifier = 0;

        let mut mouse_buttons = 0;

        for event in events {
            let Some(key) = self.get_keycode(event.row, event.col) else {
                continue;
            };

            match event.change_type {
                KeyStatusChangeType::Released(duration) => {
                    // Apply keycodes to the report
                    let mut handle_kc_released = |kc: KeyCode| match kc {
                        KeyCode::Key(kc) => {
                            if keycodes_idx < 6 {
                                keycodes[keycodes_idx] = kc as u8;
                                keycodes_idx += 1;
                            }
                        }
                        KeyCode::Special(kc) => match kc {
                            Special::MoScrl => {
                                self.scroll_mode = false;
                            }
                        },
                        _ => {}
                    };

                    match key {
                        KeyAction::Normal(kc) => {
                            handle_kc_released(kc);
                        }
                        KeyAction::Tap(kc) => {
                            let key_state =
                                &mut self.key_state[event.row as usize][event.col as usize];
                            if key_state.tapped {
                                handle_kc_released(kc);
                                key_state.tapped = false;
                            }
                        }
                        KeyAction::TapHold(tkc, hkc) => {
                            if duration.as_millis() > TAP_THRESHOLD {
                                handle_kc_released(hkc);
                            } else {
                                handle_kc_released(tkc);
                            }
                        }
                        _ => {}
                    }
                }
                KeyStatusChangeType::Pressed => {
                    let mut handle_kc_pressed = |kc: KeyCode| match kc {
                        KeyCode::Special(kc) => match kc {
                            Special::MoScrl => {
                                self.scroll_mode = true;
                            }
                        },
                        _ => {}
                    };

                    match key {
                        KeyAction::Normal(kc) => {
                            handle_kc_pressed(kc);
                        }
                        _ => {}
                    }
                }
                KeyStatusChangeType::Pressing(duration) => {
                    let mut handle_kc_pressing = |kc: KeyCode| match kc {
                        KeyCode::Key(kc) => {
                            if keycodes_idx < 6 {
                                keycodes[keycodes_idx] = kc as u8;
                                keycodes_idx += 1;
                            }
                        }
                        KeyCode::Mouse(btn) => {
                            mouse_buttons |= btn as u8;
                        }
                        KeyCode::Modifier(modi) => {
                            modifier |= modi as u8;
                        }
                        _ => {}
                    };

                    match key {
                        KeyAction::Normal(kc) => {
                            handle_kc_pressing(kc);
                        }
                        _ => {}
                    }
                }
            }
        }

        if *mouse_event != (0, 0) || mouse_buttons != 0 {
            self.layer_active[AUTO_MOUSE_LAYER] = true;
            self.auto_mouse_start = Some(now);
        } else if let Some(start) = self.auto_mouse_start {
            if now.duration_since(start).as_millis() > AUTO_MOUSE_TIME {
                self.layer_active[AUTO_MOUSE_LAYER] = false;
            }
        };

        let mouse_report = if *mouse_event == (0, 0) && mouse_buttons == 0 {
            if self.empty_mouse_sent {
                self.empty_mouse_sent = false;
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
            self.empty_mouse_sent = true;
            if self.scroll_mode {
                Some(MouseReport {
                    x: 0,
                    y: 0,
                    buttons: mouse_buttons,
                    wheel: mouse_event.1,
                    pan: mouse_event.0,
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
            if self.empty_kb_sent {
                self.empty_kb_sent = false;
                Some(KeyboardReport::default())
            } else {
                None
            }
        } else {
            self.empty_kb_sent = true;
            Some(KeyboardReport {
                keycodes,
                modifier,
                reserved: 0,
                leds: 0,
            })
        };

        StateReport {
            keyboard_report,
            mouse_report,
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
            if *key != KeyAction::None {
                return Some(*key);
            }
        }

        None
    }
}
