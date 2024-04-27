use embassy_futures::join::join3;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Channel};
use usbd_hid::descriptor::{KeyboardReport, MouseReport};

use crate::display::DISPLAY;

use super::{split::*, CoreTaskResource};

mod main_loop;
mod report;
mod split_handler;

type KeyboardReportChannel = Channel<ThreadModeRawMutex, KeyboardReport, 10>;
type MouseReportChannel = Channel<ThreadModeRawMutex, MouseReport, 10>;

pub async fn start(r: CoreTaskResource<'_>) {
    DISPLAY.set_master(true).await;

    let s2m_chan: S2mChannel = Channel::new();
    let s2m_tx = s2m_chan.sender();
    let s2m_rx = s2m_chan.receiver();

    let m2s_chan: M2sChannel = Channel::new();
    let m2s_tx = m2s_chan.sender();
    let m2s_rx = m2s_chan.receiver();

    let kb_report_chan: KeyboardReportChannel = Channel::new();
    let kb_report_tx = kb_report_chan.sender();
    let kb_report_rx = kb_report_chan.receiver();
    let mouse_report_chan: MouseReportChannel = Channel::new();
    let mouse_report_tx = mouse_report_chan.sender();
    let mouse_report_rx = mouse_report_chan.receiver();

    let (_kb_reader, kb_writer) = r.hid.keyboard.split();
    let (_mouse_reader, mouse_writer) = r.hid.mouse.split();

    join3(
        main_loop::start(main_loop::MasterMainLoopResource {
            ball: r.ball,
            scanner: r.scanner,
            s2m_rx,
            m2s_tx,
            led_controller: r.led_controller,
            hand: r.hand,
            mouse_report_tx,
            kb_report_tx,
        }),
        split_handler::start(r.split_peripherals, m2s_rx, s2m_tx),
        report::start(
            kb_report_rx,
            mouse_report_rx,
            kb_writer,
            mouse_writer,
            r.remote_wakeup_signal,
        ),
    )
    .await;
}
