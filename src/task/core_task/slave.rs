use core::fmt::Write;

use embassy_futures::join::join;
use embassy_time::Timer;

use crate::{
    display::DISPLAY,
    driver::{
        ball::Ball,
        keyboard::{pressed::Pressed, Keyboard},
    },
};

use super::split::{M2sRx, S2mTx};

/// Slave-side main task.
pub async fn start(
    mut ball: Option<Ball<'_>>,
    mut keyboard: Keyboard<'_>,
    m2s_rx: M2sRx<'_>,
    s2m_tx: S2mTx<'_>,
) {
    DISPLAY.set_master(false).await;

    let mut pressed = Pressed::new();
    loop {
        let start = embassy_time::Instant::now();

        join(
            async {
                let changed = keyboard.scan_and_update(&mut pressed).await;
                if changed {
                    let mut keys = [None; 6];

                    for (idx, (row, col)) in pressed.iter().enumerate() {
                        if idx >= keys.len() {
                            break;
                        }
                        keys[idx] = Some((row, col));
                    }

                    let mut str = heapless::String::<256>::new();
                    for (row, col) in keys.iter().flatten() {
                        write!(str, "{}:{},", row, col).unwrap();
                    }
                    DISPLAY.set_message(&str).await;

                    s2m_tx
                        .send(super::split::SlaveToMaster::Pressed { keys })
                        .await;
                }
            },
            async {
                if let Some(ball) = &mut ball {
                    if let Ok(Some(data)) = ball.read().await {
                        s2m_tx
                            .send(super::split::SlaveToMaster::Mouse {
                                // x and y are swapped
                                x: data.0,
                                y: data.1,
                            })
                            .await;
                    }
                }
            },
        )
        .await;

        DISPLAY.set_update_time(start.elapsed().as_millis()).await;

        Timer::after_millis(10).await;
    }
}
