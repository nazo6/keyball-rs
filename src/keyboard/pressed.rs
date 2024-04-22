use core::{
    array::from_fn,
    fmt::{self, Formatter},
};

use embassy_time::{Duration, Instant};

#[derive(Default)]
pub struct Key<T: Default> {
    pub press_start: Option<Instant>,
    pub data: T,
}

/// この構造体には、キーが押下されているかどうかの情報が格納される。
/// NOTE:
/// この構造体は「生の」COLを受け取るし、返す。つまり、手によるボードの反転を考慮するのはこの構造体の責務ではない。
pub struct Pressed<const COLS: usize, const ROWS: usize, T: Default> {
    state: [[Key<T>; COLS]; ROWS],
}

pub enum KeyChangeInfo {
    Pressed,
    Released(Duration),
}

impl<const COLS: usize, const ROWS: usize, T: Default> Pressed<COLS, ROWS, T> {
    pub fn new() -> Self {
        Self {
            state: from_fn(|_| from_fn(|_| Key::default())),
        }
    }
    /// Panic safety: row < ROWS, col < COLSでなければならない
    pub fn set_pressed(
        &mut self,
        pressed: bool,
        row: u8,
        col: u8,
        update_time: Instant,
    ) -> Option<KeyChangeInfo> {
        let state = &mut self.state[row as usize][col as usize];
        if let Some(time) = state.press_start {
            if pressed {
                None
            } else {
                state.press_start = None;

                Some(KeyChangeInfo::Released(update_time.duration_since(time)))
            }
        } else if pressed {
            state.press_start = Some(update_time);
            Some(KeyChangeInfo::Pressed)
        } else {
            None
        }
    }
}

impl<const COLS: usize, const ROWS: usize, T: Default> core::fmt::Debug for Pressed<COLS, ROWS, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (row, col, _) in self.iter() {
            write!(f, "{},{} ", row, col)?;
        }
        Ok(())
    }
}

/// このイテレータで返されるcolは「全体から見たcol」である。
/// つまり、右手では+7されている。
pub struct PressedIter<'a, const COLS: usize, const ROWS: usize, T: Default> {
    pressed: &'a Pressed<COLS, ROWS, T>,
    idx_row: usize,
    idx_col: usize,
}

impl<'a, const COLS: usize, const ROWS: usize, T: Default> Iterator
    for PressedIter<'a, COLS, ROWS, T>
{
    type Item = (u8, u8, &'a mut T);
    fn next(&mut self) -> Option<Self::Item> {
        for i in self.idx_row..ROWS {
            for j in self.idx_col..COLS {
                let mut key = &mut self.pressed.state[i][j];
                if key.press_start.is_some() {
                    self.idx_row = i;
                    self.idx_col = j + 1;

                    let row = i as u8;
                    let col = j as u8;

                    return Some((row, col, &mut key.data));
                }
            }
            self.idx_col = 0;
        }
        None
    }
}

impl<const COLS: usize, const ROWS: usize, T: Default> Pressed<COLS, ROWS, T> {
    pub fn iter(&self) -> PressedIter<COLS, ROWS, T> {
        PressedIter {
            pressed: self,
            idx_row: 0,
            idx_col: 0,
        }
    }
}
