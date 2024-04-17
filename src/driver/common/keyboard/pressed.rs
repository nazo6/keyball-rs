use crate::{
    constant::{COLS, LEFT_DETECT_JUMPER_KEY, ROWS},
    keyconfig::keymap::KEYMAP,
};

// Struct to store the state of the keys
pub struct Pressed {
    pub state: [[bool; COLS]; ROWS],
}

impl Pressed {
    pub fn new() -> Self {
        Self {
            state: [[false; COLS]; ROWS],
        }
    }
    pub fn set_pressed(&mut self, state: bool, row: u8, col: u8) -> bool {
        // In left side, this is always high.
        if (row, col) == LEFT_DETECT_JUMPER_KEY {
            return false;
        }

        let prev = self.state[row as usize][col as usize];
        self.state[row as usize][col as usize] = state;
        state != prev
    }

    pub fn get_keycode(&self, row: u8, col: u8) -> Option<u8> {
        if row >= ROWS as u8 || col >= COLS as u8 {
            return None;
        }
        let kc = KEYMAP[row as usize][col as usize];
        if kc == 0 {
            return None;
        }
        Some(kc)
    }
}

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
                    return Some((i as u8, j as u8));
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
