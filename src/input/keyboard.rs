use embassy_rp::gpio::{Flex, Pull};
use usbd_hid::descriptor::KeyboardReport;

use crate::{keycodes::KC_NO, keymap};

use super::KeyboardPeripherals;

pub struct Keyboard<'a> {
    rows: [Flex<'a>; 5],
    cols: [Flex<'a>; 4],
}

impl<'a> Keyboard<'a> {
    pub fn new(p: KeyboardPeripherals) -> Self {
        Self {
            rows: [
                Flex::new(p.row_0),
                Flex::new(p.row_1),
                Flex::new(p.row_2),
                Flex::new(p.row_3),
                Flex::new(p.row_4),
            ],
            cols: [
                Flex::new(p.col_0),
                Flex::new(p.col_1),
                Flex::new(p.col_2),
                Flex::new(p.col_3),
            ],
        }
    }

    /// Read the keyboard matrix and return a KeyboardReport.
    /// If no keys are pressed, returns None.
    pub async fn read(
        &mut self,
        other_side_keys: &[Option<(u8, u8)>; 6],
    ) -> Option<KeyboardReport> {
        let keys = self.read_matrix().await;

        let mut keycodes = [0; 6];
        let mut idx = 0;

        for key in other_side_keys.iter() {
            if idx >= keycodes.len() {
                break;
            }
            if let Some((row, col)) = key {
                let kc = keymap::KEYMAP[*row as usize][*col as usize];
                if kc == KC_NO {
                    continue;
                }
                keycodes[idx] = kc;
                idx += 1;
            }
        }

        for (row, col) in keys.iter() {
            if idx >= keycodes.len() {
                break;
            }
            let kc = keymap::KEYMAP[*row as usize][*col as usize + 7];
            if kc == KC_NO {
                continue;
            }
            keycodes[idx] = kc;
            idx += 1;
        }

        if idx == 0 {
            return None;
        }

        Some(KeyboardReport {
            keycodes,
            leds: 0,
            modifier: 0,
            reserved: 0,
        })
    }

    pub async fn read_matrix(&mut self) -> heapless::Vec<(u8, u8), 100> {
        let mut keys = heapless::Vec::<(u8, u8), 100>::new();

        for row in self.rows.iter_mut() {
            row.set_as_output();
            row.set_low();
        }
        for col in self.cols.iter_mut() {
            col.set_as_input();
            col.set_pull(Pull::Down);
        }

        for (i, row) in self.rows.iter_mut().enumerate() {
            row.set_high();
            row.wait_for_high().await;

            for (j, col) in self.cols.iter_mut().enumerate() {
                if col.is_high() {
                    keys.push((i as u8, j as u8)).unwrap();
                }
            }

            row.set_low();
            row.wait_for_low().await;
        }

        for row in self.rows.iter_mut() {
            row.set_as_input();
            row.set_pull(Pull::Down);
        }
        for col in self.cols.iter_mut() {
            col.set_as_output();
            col.set_low();
        }

        for (j, col) in self.cols.iter_mut().enumerate() {
            col.set_high();
            col.wait_for_high().await;

            for (i, row) in self.rows.iter_mut().enumerate() {
                if row.is_high() {
                    keys.push((i as u8, j as u8 + 3)).unwrap();
                }
            }

            col.set_low();
            col.wait_for_low().await;
        }

        keys
    }
}
