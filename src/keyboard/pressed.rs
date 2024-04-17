//! One-hand keyboard pressed state.

use core::fmt::{self, Formatter};

use crate::{
    constant::{COLS, LEFT_DETECT_JUMPER_KEY, ROWS},
    driver::keyboard::Hand,
};

// Struct to store the state of the keys
pub struct Pressed {
    /// 判定された状態のcolを使う
    pub state: [[bool; COLS]; ROWS],
    /// 右手ではcolが反転する。
    pub hand: Hand,
}

impl Pressed {
    pub fn new(hand: Hand) -> Self {
        Self {
            state: [[false; COLS]; ROWS],
            hand,
        }
    }
    /// Panic safety: row < ROWS, col < COLSでなければならない
    pub fn set_pressed(&mut self, state: bool, row: u8, col_raw: u8) -> bool {
        // In left side, this is always high.
        if (row, col_raw) == LEFT_DETECT_JUMPER_KEY {
            return false;
        }

        let col = if self.hand == Hand::Right {
            COLS - 1 - col_raw as usize
        } else {
            col_raw as usize
        };

        let prev = self.state[row as usize][col];
        self.state[row as usize][col] = state;
        state != prev
    }
}

impl core::fmt::Debug for Pressed {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (row, col) in self.iter() {
            write!(f, "{},{} ", row, col)?;
        }
        Ok(())
    }
}

/// このイテレータで返されるcolは「全体から見たcol」である。
/// つまり、右手では+7されている。
pub struct PressedIter<'a> {
    pressed: &'a Pressed,
    idx_row: usize,
    idx_col: usize,
}

impl Iterator for PressedIter<'_> {
    type Item = (u8, u8);
    fn next(&mut self) -> Option<Self::Item> {
        for i in self.idx_row..ROWS {
            for j in self.idx_col..COLS {
                if self.pressed.state[i][j] {
                    self.idx_row = i;
                    self.idx_col = j + 1;

                    let row = i as u8;
                    let col = if self.pressed.hand == Hand::Right {
                        (j + COLS) as u8
                    } else {
                        j as u8
                    };

                    return Some((row, col));
                }
            }
            self.idx_col = 0;
        }
        None
    }
}

impl Pressed {
    pub fn iter(&self) -> PressedIter {
        PressedIter {
            pressed: self,
            idx_row: 0,
            idx_col: 0,
        }
    }
}
