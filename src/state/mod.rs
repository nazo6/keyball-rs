use embassy_time::Instant;
use usbd_hid::descriptor::{KeyboardReport, MouseReport};

use crate::{
    constant::{AUTO_MOUSE_LAYER, COLS, LAYER_NUM, ROWS},
    driver::keyboard::{Hand, KeyboardScanner},
    keyboard::{
        keycode::{special::Special, KeyCode, Layer},
        pressed::KeyChangeInfo,
    },
};

use self::report::StateReport;

use super::keyboard::{keycode::KeyAction, pressed::Pressed};

pub mod report;

#[derive(Default)]
pub struct KeyState {
    tapped: bool,
}
const ALL_COLS: usize = COLS * 2;
type StatePressed = Pressed<ALL_COLS, ROWS, KeyState>;

pub struct KeyEvent {
    pub row: u8,
    pub col: u8,
    pub pressed: bool,
}

pub struct State<'a> {
    layers: [Layer; LAYER_NUM],
    layer_active: [bool; LAYER_NUM],
    master_hand: Hand,
    pressed: StatePressed,
    auto_mouse_start: Option<Instant>,
    scroll_mode: bool,
    empty_kb_sent: bool,
    empty_mouse_sent: bool,
    master_keyboard_scanner: KeyboardScanner<'a>,
}

impl<'a> State<'a> {
    pub fn new(
        layers: [Layer; LAYER_NUM],
        master_hand: Hand,
        master_keyboard_scanner: KeyboardScanner<'a>,
    ) -> Self {
        Self {
            layers,
            layer_active: [false; LAYER_NUM],
            master_hand,
            pressed: Pressed::new(),
            auto_mouse_start: None,
            scroll_mode: false,
            empty_kb_sent: false,
            empty_mouse_sent: false,
            master_keyboard_scanner,
        }
    }

    pub fn update(&mut self, slave_events: &[KeyEvent], mouse_event: (u8, u8)) -> StateReport {
        let now = Instant::now();

        let mut master_events = heapless::Vec::new();
        self.master_keyboard_scanner.scan_and_update_with_cb(
            &mut self.pressed,
            now,
            |row, col, change| {},
        );

        let (left_events, mut right_events) = if self.master_hand == Hand::Left {
            (master_events, slave_events)
        } else {
            (slave_events, master_events)
        };
        right_events
            .iter_mut()
            .map(|event| event.col = ((COLS - 1) as u8 - event.col) + COLS as u8);
        let events = master_events.iter_mut().chain(slave_events.iter_mut());

        let key_changes = events.map(|event| {
            (
                event,
                self.pressed
                    .set_pressed(event.pressed, event.row, event.col, now),
            )
        });

        for (event, change) in key_changes {
            match change {
                KeyChangeInfo::Released(_) => {
                    if let Some(key) = self.get_keycode(event.row, event.col) {
                        // disable scroll mode
                        if key == KeyAction::Normal(KeyCode::Special(Special::MoScrl)) {
                            self.scroll_mode = false;
                        }
                    }
                }
                _ => {}
            }
        }

        let mut keycodes = [0; 6];
        let mut keycodes_idx = 0;

        let mut modifier = 0;

        let mut mouse_buttons = 0;

        let mut handle_kc = |kc: KeyCode| match kc {
            KeyCode::Modifier(m) => {
                modifier |= m as u8;
            }
            KeyCode::Mouse(mb) => {
                mouse_buttons |= mb as u8;
            }
            KeyCode::Key(kc) => {
                if keycodes_idx < 6 {
                    keycodes[keycodes_idx] = kc as u8;
                    keycodes_idx += 1;
                }
            }
            KeyCode::Special(sc) => match sc {
                Special::MoScrl => {
                    self.scroll_mode = true;
                }
            },
            KeyCode::Layer(_) => todo!(),
        };

        for (row, col, key_state) in self.pressed.iter() {
            if let Some(key) = self.get_keycode(row, col) {
                match key {
                    KeyAction::None => {}
                    KeyAction::Inherit => {}
                    KeyAction::Normal(kc) => handle_kc(kc),
                    KeyAction::Tap(_) => {
                        if !key_state.tapped {
                            handle_kc(KeyCode::Key(0));
                            key_state.tapped = true;
                        }
                    }
                    KeyAction::TapHold(_, _) => todo!(),
                }
            }
        }

        if mouse_event != (0, 0) {
            self.layer_active[AUTO_MOUSE_LAYER] = true;
            self.auto_mouse_start = Some(now);
        } else if let Some(start) = self.auto_mouse_start {
            if now.duration_since(start).as_millis() > 500 {
                self.layer_active[AUTO_MOUSE_LAYER] = false;
            }
        };

        let mouse_report = if mouse_event == (0, 0) && mouse_buttons == 0 {
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
                    wheel: mouse_event.1 as i8,
                    pan: mouse_event.0 as i8,
                })
            } else {
                Some(MouseReport {
                    x: mouse_event.0 as i8,
                    y: mouse_event.1 as i8,
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
        }
    }

    fn get_keycode(&self, row: u8, col: u8) -> Option<KeyAction> {
        if row >= ROWS as u8 || col >= (COLS * 2) as u8 {
            return None;
        }

        let highest_layer = self.layer_active.iter().rposition(|&x| x).unwrap_or(0);

        for layer in (0..=highest_layer).rev() {
            let key = self.layers[layer][row as usize][col as usize];
            if key != KeyAction::None {
                return Some(key);
            }
        }

        None
    }
}
