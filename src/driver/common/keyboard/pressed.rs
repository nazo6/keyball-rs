use core::fmt::{self, Formatter};

use crate::constant::{COLS, ROWS};

pub struct Pressed([[bool; COLS]; ROWS]);

impl Pressed {
    pub fn new() -> Self {
        Self([[false; COLS]; ROWS])
    }
    /// Returns None if the key state is not changed.
    /// Returns Some(true) if the key is changed to pressed.
    /// Returns Some(false) if the key is changed to released.
    pub fn set_pressed(&mut self, pressed: bool, row: u8, col: u8) -> Option<bool> {
        let prev_pressed = &mut self.0[row as usize][col as usize];
        if prev_pressed == &pressed {
            None
        } else if !*prev_pressed {
            *prev_pressed = true;
            Some(true)
        } else {
            *prev_pressed = false;
            Some(false)
        }
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

pub struct PressedIter<'a, const COLS: usize, const ROWS: usize> {
    pressed: &'a Pressed,
    idx_row: usize,
    idx_col: usize,
}

impl<'a, const COLS: usize, const ROWS: usize> Iterator for PressedIter<'a, COLS, ROWS> {
    type Item = (u8, u8);
    fn next(&mut self) -> Option<Self::Item> {
        for i in self.idx_row..ROWS {
            for j in self.idx_col..COLS {
                let pressed = &self.pressed.0[i][j];
                if *pressed {
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
    pub fn iter(&self) -> PressedIter<COLS, ROWS> {
        PressedIter {
            pressed: self,
            idx_row: 0,
            idx_col: 0,
        }
    }
}
