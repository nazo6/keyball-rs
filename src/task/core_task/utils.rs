use embassy_futures::join::join;
use usbd_hid::descriptor::{KeyboardReport, MouseReport};

use crate::driver::{ball, keyboard};

pub async fn read_report(
    keyboard: &mut keyboard::Keyboard<'_>,
    ball: Option<&mut ball::Ball<'_>>,
    other_side_keys: &[Option<(u8, u8)>; 6],
) -> (Option<KeyboardReport>, Option<MouseReport>) {
    if let Some(ball) = ball {
        let (ball, keyboard) = join(ball.read(), keyboard.read(other_side_keys)).await;
        (keyboard, ball.unwrap())
    } else {
        let keyboard = keyboard.read(other_side_keys).await;
        (keyboard, None)
    }
}
