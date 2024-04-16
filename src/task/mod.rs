use embassy_futures::join::join3;
use embassy_sync::channel::Channel;

use crate::{device::peripherals::*, usb::UsbResource};

mod core_task;
mod led_task;
mod usb_task;

pub struct TaskPeripherals {
    pub keyboard: KeyboardPeripherals,
    pub ball: BallPeripherals,
    pub split: SplitPeripherals,
    pub led: LedPeripherals,
}

pub struct TaskResource<'a> {
    pub usb: UsbResource<'a>,
    pub peripherals: TaskPeripherals,
}

/// Starts tasks.
pub async fn start(r: TaskResource<'_>) {
    let led_ctrl_chan: led_task::LedCtrlChannel = Channel::new();
    let led_ctrl_rx = led_ctrl_chan.receiver();
    let led_ctrl_tx = led_ctrl_chan.sender();

    join3(
        core_task::start(
            r.peripherals.ball,
            r.peripherals.keyboard,
            r.peripherals.split,
            led_ctrl_tx,
            r.usb.hid,
        ),
        led_task::start(r.peripherals.led, led_ctrl_rx),
        usb_task::start(r.usb.device),
    )
    .await;
}
