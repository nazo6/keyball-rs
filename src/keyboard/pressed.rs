use core::fmt::{self, Formatter};

use crate::constant::{COLS, ROWS};

/// この構造体には、キーが押下されているかどうかの情報が格納される。
/// NOTE:
/// この構造体は「生の」COLを受け取るし、返す。つまり、手によるボードの反転を考慮するのはこの構造体の責務ではない。
pub struct Pressed {
    /// 判定された状態のcolを使う
    pub state: [[bool; COLS]; ROWS],
}

impl Pressed {
    pub fn new() -> Self {
        Self {
            state: [[false; COLS]; ROWS],
        }
    }
    /// Panic safety: row < ROWS, col < COLSでなければならない
    pub fn set_pressed(&mut self, state: bool, row: u8, col: u8) -> bool {
        let prev = self.state[row as usize][col as usize];
        self.state[row as usize][col as usize] = state;
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
                    let col = j as u8;

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
