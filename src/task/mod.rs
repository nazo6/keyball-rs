use embassy_futures::join::join3;
use embassy_sync::channel::Channel;
use embassy_usb::class::hid::State;

use crate::{
    device::{peripherals::*, usb::create_usb_driver},
    usb::{device_handler::UsbDeviceHandler, request_handler::UsbRequestHandler, UsbOpts},
};

mod core_task;
mod led_task;
mod usb_task;

pub struct TaskPeripherals {
    pub ball: BallPeripherals,
    pub keyboard: KeyboardPeripherals,
    pub split: SplitPeripherals,
    pub led: LedPeripherals,
    pub usb: UsbPeripherals,
}

/// Starts tasks.
pub async fn start(p: TaskPeripherals) {
    let led_ctrl_chan: led_task::LedCtrlChannel = Channel::new();
    let led_ctrl_rx = led_ctrl_chan.receiver();
    let led_ctrl_tx = led_ctrl_chan.sender();

    let mut device_handler = UsbDeviceHandler::new();

    // Usb keyboard and mouse
    let opts = UsbOpts {
        driver: create_usb_driver(p.usb),
        config_descriptor: &mut [0; 256],
        bos_descriptor: &mut [0; 256],
        msos_descriptor: &mut [0; 256],
        control_buf: &mut [0; 64],
        request_handler: &UsbRequestHandler {},
        device_handler: &mut device_handler,
        state_kb: &mut State::new(),
        state_mouse: &mut State::new(),
    };
    let usb = crate::usb::create_usb(opts);

    join3(
        core_task::start(p.ball, p.keyboard, p.split, led_ctrl_tx, usb.hid),
        led_task::start(p.led, led_ctrl_rx),
        usb_task::start(usb.device),
    )
    .await;
}
