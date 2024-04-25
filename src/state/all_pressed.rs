use core::{
    array::from_fn,
    fmt::{self, Formatter},
};

use embassy_time::{Duration, Instant};

use crate::constant::{COLS, ROWS};

#[derive(Debug, Default)]
pub struct KeyState {
    pub press_start: Option<Instant>,
    pub tapped: bool,
}

pub struct AllPressed {
    state: [[KeyState; COLS * 2]; ROWS],
}

#[derive(Debug)]
pub enum KeyChangeTypeWithDuration {
    Pressed,
    Released(Duration),
}

impl AllPressed {
    pub fn new() -> Self {
        Self {
            state: from_fn(|_| from_fn(|_| KeyState::default())),
        }
    }
    pub fn set_pressed(
        &mut self,
        pressed: bool,
        row: u8,
        col: u8,
        update_time: Instant,
    ) -> Option<KeyChangeTypeWithDuration> {
        if row as usize >= ROWS || col as usize >= COLS * 2 {
            return None;
        }
        let key_state = &mut self.state[row as usize][col as usize];
        if let Some(time) = key_state.press_start {
            if pressed {
                None
            } else {
                key_state.press_start = None;

                Some(KeyChangeTypeWithDuration::Released(
                    update_time.duration_since(time),
                ))
            }
        } else if pressed {
            key_state.press_start = Some(update_time);
            Some(KeyChangeTypeWithDuration::Pressed)
        } else {
            None
        }
    }
}

impl core::fmt::Debug for AllPressed {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (row, col, _) in self.iter() {
            write!(f, "{},{} ", row, col)?;
        }
        Ok(())
    }
}

pub struct PressedIter<'a> {
    pressed: &'a AllPressed,
    idx_row: usize,
    idx_col: usize,
}

impl<'a> Iterator for PressedIter<'a> {
    type Item = (u8, u8, &'a KeyState);
    fn next(&mut self) -> Option<Self::Item> {
        for i in self.idx_row..ROWS {
            for j in self.idx_col..(COLS * 2) {
                let key_state = &self.pressed.state[i][j];
                if key_state.press_start.is_some() {
                    self.idx_row = i;
                    self.idx_col = j + 1;

                    let row = i as u8;
                    let col = j as u8;

                    return Some((row, col, key_state));
                }
            }
            self.idx_col = 0;
        }
        None
    }
}

impl AllPressed {
    pub fn iter(&self) -> PressedIter {
        PressedIter {
            pressed: self,
            idx_row: 0,
            idx_col: 0,
        }
    }
}
