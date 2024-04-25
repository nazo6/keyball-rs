use core::fmt::{self, Formatter};

use embassy_time::{Duration, Instant};

use crate::{
    constant::{COLS, ROWS},
    driver::keyboard::KeyChangeEventOneHand,
};

pub struct AllPressed {
    state: [[Option<Instant>; COLS * 2]; ROWS],
}

pub struct KeyStatusUpdateEvent {
    pub row: u8,
    pub col: u8,
    pub change_type: KeyStatusChangeType,
}

#[derive(Debug, Clone, Copy)]
pub enum KeyStatusChangeType {
    // Just pressed
    Pressed,
    // Still pressing
    Pressing(Duration),
    // Released
    Released(Duration),
}

impl AllPressed {
    pub fn new() -> Self {
        Self {
            state: [[None; COLS * 2]; ROWS],
        }
    }
    pub fn compose_events<'a>(
        &mut self,
        events: impl Iterator<Item = &'a KeyChangeEventOneHand>,
        update_time: Instant,
    ) -> heapless::Vec<KeyStatusUpdateEvent, 32> {
        let mut composed = heapless::Vec::new();
        for event in events {
            if event.row as usize >= ROWS || event.col as usize >= COLS * 2 {
                continue;
            }
            let key_state = &mut self.state[event.row as usize][event.col as usize];
            match (event.pressed, &key_state) {
                (true, None) => {
                    *key_state = Some(update_time);
                    composed
                        .push(KeyStatusUpdateEvent {
                            row: event.row,
                            col: event.col,
                            change_type: KeyStatusChangeType::Pressed,
                        })
                        .ok();
                }
                (false, Some(pressed_time)) => {
                    composed
                        .push(KeyStatusUpdateEvent {
                            row: event.row,
                            col: event.col,
                            change_type: KeyStatusChangeType::Released(update_time - *pressed_time),
                        })
                        .ok();
                    *key_state = None;
                }
                (true, Some(pressed_time)) => {
                    composed
                        .push(KeyStatusUpdateEvent {
                            row: event.row,
                            col: event.col,
                            change_type: KeyStatusChangeType::Pressing(update_time - *pressed_time),
                        })
                        .ok();
                    *key_state = Some(update_time);
                }
                (false, None) => {}
            }
        }

        composed
    }
}

impl core::fmt::Debug for AllPressed {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (row, col) in self.iter() {
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
    type Item = (u8, u8);
    fn next(&mut self) -> Option<Self::Item> {
        for i in self.idx_row..ROWS {
            for j in self.idx_col..(COLS * 2) {
                let key_state = &self.pressed.state[i][j];
                if key_state.is_some() {
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

impl AllPressed {
    pub fn iter(&self) -> PressedIter {
        PressedIter {
            pressed: self,
            idx_row: 0,
            idx_col: 0,
        }
    }
}
