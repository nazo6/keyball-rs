use embassy_time::Instant;
use usbd_hid::descriptor::{KeyboardReport, MouseReport};

use crate::{
    constant::{AUTO_MOUSE_LAYER, AUTO_MOUSE_TIME, COLS, LAYER_NUM, ROWS},
    driver::keyboard::{Hand, KeyChangeEvent},
    keyboard::keycode::{special::Special, KeyCode, Layer},
};

use self::{
    all_pressed::{AllPressed, KeyChangeTypeWithDuration},
    report::StateReport,
};

use super::keyboard::keycode::KeyAction;

mod all_pressed;
pub mod report;

pub struct State {
    master_hand: Hand,

    layers: [Layer; LAYER_NUM],
    layer_active: [bool; LAYER_NUM],

    pressed: AllPressed,

    auto_mouse_start: Option<Instant>,
    scroll_mode: bool,

    empty_kb_sent: bool,
    empty_mouse_sent: bool,
}

impl State {
    pub fn new(layers: [Layer; LAYER_NUM], master_hand: Hand) -> Self {
        Self {
            layers,
            layer_active: [false; LAYER_NUM],
            master_hand,
            pressed: AllPressed::new(),
            auto_mouse_start: None,
            scroll_mode: false,
            empty_kb_sent: false,
            empty_mouse_sent: false,
        }
    }

    pub fn update(
        &mut self,
        master_events: &mut [KeyChangeEvent],
        slave_events: &mut [KeyChangeEvent],
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

        let events = right_events.iter_mut().chain(left_events.iter_mut());

        for event in events {
            let change = self
                .pressed
                .set_pressed(event.pressed, event.row, event.col, now);
            if let Some(KeyChangeTypeWithDuration::Released(_)) = change {
                if let Some(key) = self.get_keycode(event.row, event.col) {
                    // disable scroll mode
                    if *key == KeyAction::Normal(KeyCode::Special(Special::MoScrl)) {
                        self.scroll_mode = false;
                    }
                }
            }
        }

        let mut keycodes = [0; 6];
        let mut keycodes_idx = 0;

        let mut modifier = 0;

        let mut mouse_buttons = 0;

        let mut handle_kc = |kc: &KeyCode| match kc {
            KeyCode::Modifier(m) => {
                modifier |= *m as u8;
            }
            KeyCode::Mouse(mb) => {
                mouse_buttons |= *mb as u8;
            }
            KeyCode::Key(kc) => {
                if keycodes_idx < 6 {
                    keycodes[keycodes_idx] = *kc as u8;
                    keycodes_idx += 1;
                }
            }
            KeyCode::Special(sc) => match sc {
                Special::MoScrl => {
                    // *scroll_mode = true;
                }
            },
            KeyCode::Layer(_) => todo!(),
        };

        for (row, col, key_state) in self.pressed.iter() {
            let key = self.get_keycode(row, col);
            if let Some(key) = key {
                match key {
                    KeyAction::None => {}
                    KeyAction::Inherit => {}
                    KeyAction::Normal(kc) => handle_kc(kc),
                    KeyAction::Tap(kc) => {
                        if !key_state.tapped {
                            handle_kc(kc);
                            // key_state.tapped = true;
                        }
                    }
                    KeyAction::TapHold(_, _) => todo!(),
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

    fn get_keycode(&self, row: u8, col: u8) -> Option<&KeyAction> {
        if row >= ROWS as u8 || col >= (COLS * 2) as u8 {
            return None;
        }

        let highest_layer = self.layer_active.iter().rposition(|&x| x).unwrap_or(0);

        for layer in (0..=highest_layer).rev() {
            let key = &self.layers[layer][row as usize][col as usize];
            if *key != KeyAction::None {
                return Some(key);
            }
        }

        None
    }
}
