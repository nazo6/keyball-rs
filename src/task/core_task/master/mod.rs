use embassy_futures::join::join;
use embassy_sync::channel::Channel;

use crate::display::DISPLAY;

use super::{split::*, CoreTaskResource};

mod main_loop;
mod split_handler;

pub async fn start(r: CoreTaskResource<'_>) {
    DISPLAY.set_master(true).await;

    let s2m_chan: S2mChannel = Channel::new();
    let s2m_tx = s2m_chan.sender();
    let s2m_rx = s2m_chan.receiver();

    let m2s_chan: M2sChannel = Channel::new();
    let m2s_tx = m2s_chan.sender();
    let m2s_rx = m2s_chan.receiver();

    join(
        main_loop::start(main_loop::MasterMainLoopResource {
            ball: r.ball,
            scanner: r.scanner,
            s2m_rx,
            m2s_tx,
            led_controller: r.led_controller,
            hid: r.hid,
            remote_wakeup_signal: r.remote_wakeup_signal,
            hand: r.hand,
        }),
        split_handler::start(r.split_peripherals, m2s_rx, s2m_tx),
    )
    .await;
}
