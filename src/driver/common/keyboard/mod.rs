use crate::device::gpio::{Flex, Pull};
use crate::device::peripherals::KeyboardPeripherals;

use self::pressed::Pressed;

pub mod pressed;

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

    // Returns true if value is changed.
    pub async fn scan_and_update(&mut self, state: &mut Pressed) -> bool {
        let mut changed = false;

        // row -> col scan
        {
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

                for (j, col) in self.cols.iter().enumerate() {
                    changed |= state.set_pressed(col.is_high(), i as u8, j as u8);
                }

                row.set_low();
                row.wait_for_low().await;
            }
        }

        // col -> row scan
        {
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
                    changed |= state.set_pressed(row.is_high(), i as u8, j as u8 + 3);
                }

                col.set_low();
                col.wait_for_low().await;
            }
        }

        changed
    }
}
