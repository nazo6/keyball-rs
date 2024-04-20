use embassy_futures::join::join3;
use embassy_sync::signal::Signal;
use embassy_usb::class::hid::State;

use crate::{
    device::{peripherals::*, usb::create_usb_driver},
    driver::keyboard::KeyboardScanner,
    usb::{device_handler::UsbDeviceHandler, request_handler::UsbRequestHandler, UsbOpts},
};

use self::usb_task::RemoteWakeupSignal;

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
    let led_ctrl: led_task::LedCtrl = Signal::new();

    let remote_wakeup_signal: RemoteWakeupSignal = Signal::new();

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

    let keyboard_scanner = KeyboardScanner::new(p.keyboard).await;

    crate::DISPLAY.set_hand(keyboard_scanner.hand).await;

    join3(
        led_task::start(led_task::LedTaskResource {
            peripherals: p.led,
            led_ctrl: &led_ctrl,
            hand: keyboard_scanner.hand,
        }),
        core_task::start(core_task::CoreTaskResource {
            ball_peripherals: p.ball,
            split_peripherals: p.split,
            scanner: keyboard_scanner,
            led_controller: &led_ctrl,
            hid: usb.hid,
            remote_wakeup_signal: &remote_wakeup_signal,
        }),
        usb_task::start(usb_task::UsbTaskResource {
            device: usb.device,
            signal: &remote_wakeup_signal,
        }),
    )
    .await;
}
