use embassy_futures::join::join3;
use embassy_sync::channel::Channel;

use crate::display::DISPLAY;

use super::{split::*, CoreTaskResource};

mod main_loop;
mod split_handler;

/// Slave-side main task.
pub async fn start(r: CoreTaskResource<'_>) {
    DISPLAY.set_master(false).await;

    let s2m_chan: S2mChannel = Channel::new();
    let s2m_tx = s2m_chan.sender();
    let s2m_rx = s2m_chan.receiver();

    let m2s_chan: M2sChannel = Channel::new();
    let m2s_tx = m2s_chan.sender();
    let m2s_rx = m2s_chan.receiver();

    join3(
        main_loop::start(main_loop::SlaveMainLoopResource {
            ball: r.ball,
            scanner: r.scanner,
            s2m_tx,
        }),
        split_handler::start(r.split_peripherals, m2s_tx, s2m_rx),
        async {
            loop {
                let data = m2s_rx.receive().await;
                match data {
                    super::split::MasterToSlave::Led(ctrl) => {
                        r.led_controller.signal(ctrl);
                    }
                    super::split::MasterToSlave::Message(_) => {}
                }
            }
        },
    )
    .await;
}
