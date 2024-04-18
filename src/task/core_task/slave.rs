use embassy_futures::join::join;
use embassy_time::Timer;

use crate::{
    constant::MIN_SCAN_INTERVAL,
    display::DISPLAY,
    driver::{ball::Ball, keyboard::KeyboardScanner},
    keyboard::pressed::Pressed,
};

use super::split::{M2sRx, S2mTx, SlaveToMaster};

/// Slave-side main task.
pub async fn start(
    mut ball: Option<Ball<'_>>,
    mut scanner: KeyboardScanner<'_>,
    m2s_rx: M2sRx<'_>,
    s2m_tx: S2mTx<'_>,
) {
    DISPLAY.set_master(false).await;

    let mut pressed = Pressed::new();
    loop {
        let start = embassy_time::Instant::now();

        join(
            async {
                let mut changes = heapless::Vec::<SlaveToMaster, 6>::new();
                scanner
                    .scan_and_update_with_cb(&mut pressed, |row, col, state| {
                        if state {
                            changes
                                .push(SlaveToMaster::Pressed(row as u8, col as u8))
                                .ok();
                        } else {
                            changes
                                .push(SlaveToMaster::Released(row as u8, col as u8))
                                .ok();
                        }
                    })
                    .await;

                for change in changes {
                    crate::utils::print!("S2M: {:?}\n", change);
                    s2m_tx.send(change).await;
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
