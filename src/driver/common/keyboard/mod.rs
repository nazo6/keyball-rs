use crate::constant::{COL_PIN_NUM, LEFT_DETECT_JUMPER_KEY, SCAN_PIN_NUM};
use crate::device::gpio::{Flex, Pull};
use crate::device::peripherals::KeyboardPeripherals;

mod hand;
mod pressed;
pub use hand::Hand;

use self::pressed::Pressed;

#[derive(Debug)]
pub struct KeyChangeEventOneHand {
    pub col: u8,
    pub row: u8,
    pub pressed: bool,
}

pub struct KeyboardScanner<'a> {
    rows: [Flex<'a>; SCAN_PIN_NUM],
    cols: [Flex<'a>; COL_PIN_NUM],
    pub pressed: Pressed,
}

impl<'a> KeyboardScanner<'a> {
    /// Detect the hand and initialize the scanner.
    pub async fn new(p: KeyboardPeripherals) -> Self {
        let rows = [
            Flex::new(p.row_0),
            Flex::new(p.row_1),
            Flex::new(p.row_2),
            Flex::new(p.row_3),
            Flex::new(p.row_4),
        ];
        let cols = [
            Flex::new(p.col_0),
            Flex::new(p.col_1),
            Flex::new(p.col_2),
            Flex::new(p.col_3),
        ];
        Self {
            rows,
            cols,
            pressed: Pressed::new(),
        }
    }

    pub async fn scan(&mut self) -> heapless::Vec<KeyChangeEventOneHand, 16> {
        let mut events = heapless::Vec::new();
        self.scan_with_cb(|e| {
            events.push(e).ok();
        })
        .await;
        // crate::print!("{:?}                     ", self.pressed);
        events
    }

    async fn scan_with_cb(&mut self, mut cb: impl FnMut(KeyChangeEventOneHand)) {
        // col -> row scan
        {
            for row in self.rows.iter_mut() {
                row.set_as_input();
                row.set_pull(Pull::Down);
            }

            for (j, col) in self.cols.iter_mut().enumerate() {
                // col -> rowスキャンではcol=3は該当キーなし
                if j == 3 {
                    continue;
                }

                col.set_as_output();
                col.set_low();
                col.set_high();
                col.wait_for_high().await;

                for (i, row) in self.rows.iter_mut().enumerate() {
                    if let Some(change) = self.pressed.set_pressed(row.is_high(), i as u8, j as u8)
                    {
                        cb(KeyChangeEventOneHand {
                            row: i as u8,
                            col: j as u8,
                            pressed: change,
                        });
                    }
                }
                col.set_low();
                col.wait_for_low().await;
                col.set_as_input();
            }
        }

        // row -> col scan
        {
            for col in self.cols.iter_mut() {
                col.set_as_input();
                col.set_pull(Pull::Down);
            }

            for (i, row) in self.rows.iter_mut().enumerate() {
                row.set_as_output();
                row.set_low();
                row.set_high();
                row.wait_for_high().await;

                for (j, col) in self.cols.iter_mut().enumerate() {
                    // In left side, this is always high.
                    if (i, j + 3) == LEFT_DETECT_JUMPER_KEY {
                        continue;
                    }

                    if let Some(change) =
                        self.pressed
                            .set_pressed(col.is_high(), i as u8, (j + 3) as u8)
                    {
                        cb(KeyChangeEventOneHand {
                            row: i as u8,
                            col: (j + 3) as u8,
                            pressed: change,
                        })
                    }
                }

                row.set_low();
                row.wait_for_low().await;
                row.set_as_input();
            }
        }
    }
}
