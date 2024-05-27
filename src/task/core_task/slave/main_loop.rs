use embassy_futures::join::join;
use embassy_time::Timer;

use crate::{
    config::{MIN_KB_SCAN_INTERVAL, MIN_MOUSE_SCAN_INTERVAL},
    driver::{ball::Ball, keyboard::KeyboardScanner},
};

use super::super::split::*;

pub struct SlaveMainLoopResource<'a> {
    pub ball: Option<Ball<'a>>,
    pub scanner: KeyboardScanner<'a>,
    pub s2m_tx: S2mTx<'a>,
}

pub(super) async fn start(
    SlaveMainLoopResource {
        mut ball,
        mut scanner,
        s2m_tx,
    }: SlaveMainLoopResource<'_>,
) {
    join(
        async {
            if let Some(ball) = &mut ball {
                loop {
                    let start = embassy_time::Instant::now();

                    if let Ok(Some(data)) = ball.read().await {
                        let e = SlaveToMaster::Mouse {
                            // x and y are swapped
                            x: data.0,
                            y: data.1,
                        };
                        s2m_tx.send(e).await;
                    }

                    let took = start.elapsed();
                    if took < MIN_MOUSE_SCAN_INTERVAL {
                        Timer::after(MIN_MOUSE_SCAN_INTERVAL - took).await;
                    }
                }
            }
        },
        async {
            loop {
                let start = embassy_time::Instant::now();

                let key_events = scanner.scan().await;

                for event in key_events {
                    let event = if event.pressed {
                        SlaveToMaster::Pressed(event.row, event.col)
                    } else {
                        SlaveToMaster::Released(event.row, event.col)
                    };

                    s2m_tx.send(event).await;
                }

                let took = start.elapsed();
                if took < MIN_KB_SCAN_INTERVAL {
                    Timer::after(MIN_KB_SCAN_INTERVAL - took).await;
                }
            }
        },
    )
    .await;
}
