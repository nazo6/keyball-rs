use crate::constant::{LEFT_DETECT_JUMPER_KEY, SCAN_COLS, SCAN_ROWS};
use crate::device::gpio::{Flex, Pull};
use crate::device::peripherals::KeyboardPeripherals;
use crate::keyboard::pressed::Pressed;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    Left,
    Right,
}

pub struct Keyboard<'a> {
    rows: [Flex<'a>; SCAN_ROWS],
    cols: [Flex<'a>; SCAN_COLS],
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

    pub async fn get_hand(&mut self) -> Hand {
        if LEFT_DETECT_JUMPER_KEY.1 >= 4 {
            let row = &mut self.rows[LEFT_DETECT_JUMPER_KEY.0 as usize];
            let col = &mut self.cols[(LEFT_DETECT_JUMPER_KEY.1 - 3) as usize];

            col.set_as_input();
            col.set_pull(Pull::Down);

            row.set_as_output();
            row.set_high();
            row.wait_for_high().await;

            if col.is_high() {
                Hand::Left
            } else {
                Hand::Right
            }
        } else {
            panic!("Invalid left detect jumper config");
        }
    }

    // Returns true if value is changed.
    pub async fn scan_and_update(&mut self, state: &mut Pressed) -> bool {
        let mut changed = false;

        // col -> row scan
        {
            for (j, col) in self.cols.iter_mut().enumerate() {
                if j == 3 {
                    continue;
                }

                for row in self.rows.iter_mut() {
                    row.set_as_input();
                    row.set_pull(Pull::Down);
                }

                col.set_as_output();
                col.set_low();
                col.set_high();
                col.wait_for_high().await;

                for (i, row) in self.rows.iter_mut().enumerate() {
                    changed |= state.set_pressed(row.is_high(), i as u8, j as u8);
                }

                col.set_low();
                col.wait_for_low().await;
            }
        }

        // row -> col scan
        {
            for (i, row) in self.rows.iter_mut().enumerate() {
                for col in self.cols.iter_mut() {
                    col.set_as_input();
                    col.set_pull(Pull::Down);
                }

                row.set_as_output();
                row.set_low();
                row.set_high();
                row.wait_for_high().await;

                for (j, col) in self.cols.iter_mut().enumerate() {
                    changed |= state.set_pressed(col.is_high(), i as u8, (j + 3) as u8);
                }

                row.set_low();
                row.wait_for_low().await;
            }
        }

        changed
    }
}
