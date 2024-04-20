use embassy_futures::select::{select, Either};
use embassy_time::Timer;

use crate::{
    constant::SPLIT_USB_TIMEOUT,
    driver::{ball::Ball, keyboard::KeyboardScanner},
    usb::Hid,
};

use super::{led_task::LedCtrl, usb_task::RemoteWakeupSignal, SplitPeripherals};

mod master;
mod slave;

mod split;

pub struct CoreTaskResource<'a> {
    pub split_peripherals: SplitPeripherals,
    pub ball: Option<Ball<'a>>,
    pub scanner: KeyboardScanner<'a>,
    pub led_controller: &'a LedCtrl,
    pub hid: Hid<'a>,
    pub remote_wakeup_signal: &'a RemoteWakeupSignal,
}

pub async fn start(mut r: CoreTaskResource<'_>) {
    #[cfg(feature = "force-master")]
    let is_master = true;

    #[cfg(feature = "force-slave")]
    let is_master = false;

    // VBUS detection is not available for ProMicro RP2040, so USB communication is used to determine master/slave.
    // This is same as SPLIT_USB_DETECT in QMK.
    // let is_master = match select(
    //     Timer::after_millis(SPLIT_USB_TIMEOUT),
    // )
    // .await
    // {
    //     Either::First(_) => true,
    //     Either::Second(_) => false,
    // };

    r.hid.keyboard.ready().await;
    let is_master = true;
    crate::DISPLAY.set_master(is_master).await;

    if is_master {
        master::start(r).await;
    } else {
        slave::start(r).await;
    }
}
