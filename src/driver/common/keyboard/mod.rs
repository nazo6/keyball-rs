use embassy_time::Instant;

use crate::constant::{LEFT_DETECT_JUMPER_KEY, SCAN_COLS, SCAN_ROWS};
use crate::device::gpio::{Flex, Pull};
use crate::device::peripherals::KeyboardPeripherals;
use crate::keyboard::pressed::{KeyChangeInfo, Pressed};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    Left,
    Right,
}

impl Hand {
    /// Read the jumper to detect the hand.
    pub async fn read<'a>(
        rows: &mut [Flex<'a>; SCAN_ROWS],
        cols: &mut [Flex<'a>; SCAN_COLS],
    ) -> Self {
        if LEFT_DETECT_JUMPER_KEY.1 >= 4 {
            let row = &mut rows[LEFT_DETECT_JUMPER_KEY.0];
            let col = &mut cols[LEFT_DETECT_JUMPER_KEY.1 - 3];

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
}

pub struct KeyboardScanner<'a> {
    rows: [Flex<'a>; SCAN_ROWS],
    cols: [Flex<'a>; SCAN_COLS],
    pub hand: Hand,
}

impl<'a> KeyboardScanner<'a> {
    /// Detect the hand and initialize the scanner.
    pub async fn new(p: KeyboardPeripherals) -> Self {
        let mut rows = [
            Flex::new(p.row_0),
            Flex::new(p.row_1),
            Flex::new(p.row_2),
            Flex::new(p.row_3),
            Flex::new(p.row_4),
        ];
        let mut cols = [
            Flex::new(p.col_0),
            Flex::new(p.col_1),
            Flex::new(p.col_2),
            Flex::new(p.col_3),
        ];
        let hand = Hand::read(&mut rows, &mut cols).await;
        Self { rows, cols, hand }
    }

    // Returns true if value is changed.
    pub async fn scan_and_update<const COLS: usize, const ROWS: usize, T: Default>(
        &mut self,
        state: &mut Pressed<COLS, ROWS, T>,
        time: Instant,
    ) -> bool {
        let mut changed = false;
        self.scan_and_update_with_cb(state, time, |_, _, _| {
            changed = true;
        })
        .await;
        changed
    }

    pub async fn scan_and_update_with_cb<const COLS: usize, const ROWS: usize, T: Default>(
        &mut self,
        pressed: &mut Pressed<COLS, ROWS, T>,
        time: Instant,
        mut cb: impl FnMut(usize, usize, KeyChangeInfo),
    ) {
        // col -> row scan
        {
            for (j, col) in self.cols.iter_mut().enumerate() {
                // col -> rowスキャンではcol=3は該当キーなし
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
                    let state = row.is_high();
                    if let Some(change) = pressed.set_pressed(state, i as u8, j as u8, time) {
                        cb(i, j, change);
                    }
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
                    // In left side, this is always high.
                    if (i, j) == LEFT_DETECT_JUMPER_KEY {
                        continue;
                    }

                    let state = col.is_high();

                    if let Some(change) = pressed.set_pressed(state, i as u8, (j + 3) as u8, time) {
                        cb(i, j + 3, change);
                    }
                }

                row.set_low();
                row.wait_for_low().await;
            }
        }
    }
}
