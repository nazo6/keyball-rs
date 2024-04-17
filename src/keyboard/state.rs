use usbd_hid::descriptor::KeyboardReport;

use crate::constant::{COLS, LAYER_NUM, ROWS};

use super::{keycode::Keycode, pressed::Pressed};

pub type Layer = [[Keycode; COLS * 2]; ROWS];

pub struct KeyboardState {
    layers: [Layer; LAYER_NUM],
    layer_active: [bool; LAYER_NUM],
}

impl KeyboardState {
    pub fn new(layers: [Layer; LAYER_NUM]) -> Self {
        Self {
            layers,
            layer_active: [false; LAYER_NUM],
        }
    }

    /// Calculate the keycodes from the pressed state and return the KeyboardReport.
    /// Also update the layer_active.
    pub fn update_and_report(
        &mut self,
        master_pressed: &Pressed,
        slave_pressed: &[Option<(u8, u8)>; 6],
    ) -> (KeyboardReport, bool) {
        let mut keycodes = [0; 6];
        let mut keycodes_idx = 0;
        let mut modifier = 0;
        let mut empty_report = true;

        let mut handle_kc = |kc: Option<Keycode>| {
            if let Some(kc) = kc {
                match kc {
                    Keycode::Key(key) => {
                        if keycodes_idx < 6 {
                            keycodes[keycodes_idx] = key as u8;
                            keycodes_idx += 1;
                        }
                    }
                    Keycode::Modifier(key) => {
                        modifier |= key as u8;
                    }
                    Keycode::Mouse(_) => {}
                    Keycode::Special(_) => {}
                }
            }
        };

        for (row, col) in master_pressed.iter() {
            handle_kc(self.get_keycode(row, col));
        }

        for (row, col) in slave_pressed.iter().flatten() {
            handle_kc(self.get_keycode(*row, *col));
        }

        if keycodes_idx > 0 || modifier != 0 {
            empty_report = false;
        }

        (
            KeyboardReport {
                keycodes,
                leds: 0,
                modifier,
                reserved: 0,
            },
            empty_report,
        )
    }

    fn get_keycode(&self, row: u8, col: u8) -> Option<Keycode> {
        if row >= ROWS as u8 || col >= (COLS * 2) as u8 {
            return None;
        }
        Some(self.layers[0][row as usize][col as usize])
    }
}
