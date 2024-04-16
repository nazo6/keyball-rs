use embassy_time::Timer;

use crate::{
    display::DISPLAY,
    driver::{ball::Ball, keyboard::Keyboard},
};

use super::split::{M2sRx, S2mTx};

pub async fn main_slave_task(
    mut ball: Option<Ball<'_>>,
    mut keyboard: Keyboard<'_>,
    m2s_rx: M2sRx<'_>,
    s2m_tx: S2mTx<'_>,
) {
    DISPLAY.lock().await.as_mut().unwrap().draw_text("slave");

    let mut pressed_keys_prev = [None; 6];
    loop {
        let pressed = keyboard.read_matrix().await;
        let mut pressed_keys = [None; 6];
        for (i, (row, col)) in pressed.iter().enumerate() {
            if i >= pressed_keys.len() {
                break;
            }
            pressed_keys[i] = Some((*row, *col));
        }
        if pressed_keys != pressed_keys_prev {
            s2m_tx
                .send(super::split::SlaveToMaster::Pressed { keys: pressed_keys })
                .await;
            pressed_keys_prev = pressed_keys;
        }

        Timer::after_millis(10).await;
    }
}
