use crate::config::LEFT_DETECT_JUMPER_KEY;
use crate::device::gpio::Pull;

use super::KeyboardScanner;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    Left,
    Right,
}

impl KeyboardScanner<'_> {
    /// Detect the hand.
    pub async fn hand<'a>(&mut self) -> Hand {
        if LEFT_DETECT_JUMPER_KEY.1 >= 4 {
            let row = &mut self.rows[LEFT_DETECT_JUMPER_KEY.0];
            let col = &mut self.cols[LEFT_DETECT_JUMPER_KEY.1 - 3];

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
