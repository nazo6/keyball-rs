use embassy_futures::join::join;
use embassy_time::Timer;

use crate::{
    constant::MIN_SCAN_INTERVAL,
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
    loop {
        let start = embassy_time::Instant::now();

        join(
            async {
                let key_events = scanner.scan().await;

                for event in key_events {
                    let event = if event.pressed {
                        SlaveToMaster::Pressed(event.row, event.col)
                    } else {
                        SlaveToMaster::Released(event.row, event.col)
                    };

                    crate::print!("S2M: {:?}\n", event);

                    s2m_tx.send(event).await;
                }
            },
            async {
                if let Some(ball) = &mut ball {
                    if let Ok(Some(data)) = ball.read().await {
                        let e = SlaveToMaster::Mouse {
                            // x and y are swapped
                            x: data.0,
                            y: data.1,
                        };
                        crate::print!("S2M: {:?}\n", e);
                        s2m_tx.send(e).await;
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
