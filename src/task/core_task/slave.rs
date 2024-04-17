use embassy_futures::join::join;
use embassy_time::Timer;

use crate::{
    constant::MIN_SCAN_INTERVAL,
    display::DISPLAY,
    driver::{ball::Ball, keyboard::Keyboard},
    keyboard::pressed::Pressed,
    utils::print,
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

    let hand = keyboard.get_hand().await;
    DISPLAY.set_hand(hand).await;
    let mut pressed = Pressed::new(hand);
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

        let took = start.elapsed().as_millis();
        if took < MIN_SCAN_INTERVAL {
            Timer::after_millis(MIN_SCAN_INTERVAL - took).await;
        }
    }
}
