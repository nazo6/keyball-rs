use core::sync::atomic::Ordering;

use embassy_futures::join::join;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Receiver};
use embassy_usb::{class::hid::HidWriter, driver::Driver};
use usbd_hid::descriptor::{KeyboardReport, MouseReport};

use crate::{task::usb_task::RemoteWakeupSignal, usb::SUSPENDED};

pub async fn start<'a, 'b, D: Driver<'b>, const N: usize>(
    kb_report_rx: Receiver<'a, ThreadModeRawMutex, KeyboardReport, 10>,
    mouse_report_rx: Receiver<'a, ThreadModeRawMutex, MouseReport, 10>,
    mut keyboard_writer: HidWriter<'b, D, N>,
    mut mouse_writer: HidWriter<'b, D, N>,
    remote_wakeup_signal: &'a RemoteWakeupSignal,
) {
    join(
        async {
            loop {
                let mouse_report = mouse_report_rx.receive().await;
                let _ = mouse_writer.write_serialize(&mouse_report).await;
            }
        },
        async {
            loop {
                let kb_report = kb_report_rx.receive().await;
                let _ = keyboard_writer.write_serialize(&kb_report).await;
                if SUSPENDED.load(Ordering::Relaxed) {
                    remote_wakeup_signal.signal(());
                }
            }
        },
    )
    .await;
}
