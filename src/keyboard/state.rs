use usbd_hid::descriptor::KeyboardReport;

use crate::{
    constant::{COLS, LAYER_NUM, ROWS},
    driver::keyboard::Hand,
};

use super::{
    keycode::{self, layer, Keycode},
    pressed::Pressed,
};

pub type Layer = [[Keycode; COLS * 2]; ROWS];

pub struct KeyStateReport {
    pub keyboard_report: KeyboardReport,
    pub empty_keyboard_report: bool,
    pub mouse_button: u8,
    pub highest_layer: usize,
}

pub struct KeyboardState {
    layers: [Layer; LAYER_NUM],
    layer_active: [bool; LAYER_NUM],
    master_hand: Hand,
}

impl KeyboardState {
    pub fn new(layers: [Layer; LAYER_NUM], master_hand: Hand) -> Self {
        Self {
            layers,
            layer_active: [false; LAYER_NUM],
            master_hand,
        }
    }

    /// Calculate the keycodes from the pressed state and return the KeyboardReport.
    /// Also update the layer_active.
    pub fn update_and_report(
        &mut self,
        master_pressed: &Pressed,
        slave_pressed: &Pressed,
    ) -> KeyStateReport {
        let mut keycodes = [0; 6];
        let mut keycodes_idx = 0;

        let mut keyboard_modifier = 0;
        let mut mouse_button = 0;

        let mut empty_keyboard_report = true;

        let mut handle_kc = |kc: Option<Keycode>, layer_active: &mut [bool; LAYER_NUM]| {
            if let Some(kc) = kc {
                match kc {
                    Keycode::Key(key) => {
                        if keycodes_idx < 6 {
                            keycodes[keycodes_idx] = key as u8;
                            keycodes_idx += 1;
                        }
                    }
                    Keycode::Modifier(key) => {
                        keyboard_modifier |= key as u8;
                    }
                    Keycode::Mouse(key) => {
                        mouse_button |= key as u8;
                    }
                    Keycode::Special(_) => {}
                    Keycode::Layer(layer_cmd) => match layer_cmd {
                        layer::Layer::Move(layer) => {
                            (0..LAYER_NUM).for_each(|i| {
                                layer_active[i] = false;
                            });
                            layer_active[layer] = true;
                        }
                        layer::Layer::Toggle(..) => {}
                    },
                }
            }
        };

        // COLSを足したり引いたりしてるのは、右手側ではcolが逆転しており、
        // さらにkeymap配列で+COLSした位置にあるため。

        for (row, col) in master_pressed.iter() {
            let col = if self.master_hand == Hand::Right {
                ((COLS - 1) as u8 - col) + COLS as u8
            } else {
                col
            };
            handle_kc(self.get_keycode(row, col), &mut self.layer_active);
        }

        for (row, col) in slave_pressed.iter() {
            let col = if self.master_hand == Hand::Right {
                col
            } else {
                ((COLS - 1) as u8 - col) + COLS as u8
            };
            handle_kc(self.get_keycode(row, col), &mut self.layer_active);
        }

        if keycodes_idx > 0 || keyboard_modifier != 0 {
            empty_keyboard_report = false;
        }

        KeyStateReport {
            keyboard_report: KeyboardReport {
                keycodes,
                leds: 0,
                modifier: keyboard_modifier,
                reserved: 0,
            },
            empty_keyboard_report,
            mouse_button,
            highest_layer: self.layer_active.iter().rposition(|&x| x).unwrap_or(0),
        }
    }

    fn get_keycode(&self, row: u8, col: u8) -> Option<Keycode> {
        if row >= ROWS as u8 || col >= (COLS * 2) as u8 {
            return None;
        }

        let highest_layer = self.layer_active.iter().rposition(|&x| x).unwrap_or(0);

        for layer in (0..=highest_layer).rev() {
            let key = self.layers[layer][row as usize][col as usize];
            if key != Keycode::Special(keycode::special::Special::Inherit) {
                return Some(key);
            }
        }

        None
    }
}
