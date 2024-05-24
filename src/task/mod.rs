use embassy_futures::join::{join, join3};
use embassy_sync::signal::Signal;
use embassy_usb::class::hid::State;

use crate::{
    device::{peripherals::*, usb::create_usb_driver},
    driver::{ball::Ball, keyboard::KeyboardScanner},
    usb::{device_handler::UsbDeviceHandler, request_handler::UsbRequestHandler, UsbOpts},
};

use self::usb_task::RemoteWakeupSignal;

mod core_task;
mod led_task;
mod temperature_task;
mod usb_task;

pub struct TaskPeripherals {
    pub ball: BallPeripherals,
    pub keyboard: KeyboardPeripherals,
    pub split: SplitPeripherals,
    pub led: LedPeripherals,
    pub usb: UsbPeripherals,
    pub temp: TemperaturePeripherals,
}

/// Starts tasks.
pub async fn start(p: TaskPeripherals) {
    // Setup LED signal
    let led_controller: led_task::LedCtrl = Signal::new();

    // Setup remote wakeup signal
    let remote_wakeup_signal: RemoteWakeupSignal = Signal::new();

    // Setup USB
    let mut device_handler = UsbDeviceHandler::new();
    let opts = UsbOpts {
        driver: create_usb_driver(p.usb),
        config_descriptor: &mut [0; 256],
        bos_descriptor: &mut [0; 256],
        msos_descriptor: &mut [0; 256],
        control_buf: &mut [0; 64],
        kb_request_handler: &mut UsbRequestHandler {},
        mouse_request_handler: &mut UsbRequestHandler {},
        mkb_request_handler: &mut UsbRequestHandler {},
        device_handler: &mut device_handler,
        state_kb: &mut State::new(),
        state_mouse: &mut State::new(),
        state_media_key: &mut State::new(),
    };
    let usb = crate::usb::create_usb(opts);

    join(
        usb_task::start(usb_task::UsbTaskResource {
            device: usb.device,
            signal: &remote_wakeup_signal,
        }),
        async {
            let mut scanner = KeyboardScanner::new(p.keyboard).await;
            let hand = scanner.hand().await;
            crate::DISPLAY.set_hand(hand).await;

            join3(
                led_task::start(led_task::LedTaskResource {
                    peripherals: p.led,
                    led_ctrl: &led_controller,
                    hand,
                }),
                temperature_task::start(p.temp),
                async {
                    let ball = Ball::init(p.ball).await.ok();
                    crate::DISPLAY.set_mouse(ball.is_some()).await;
                    core_task::start(core_task::CoreTaskResource {
                        ball,
                        split_peripherals: p.split,
                        scanner,
                        led_controller: &led_controller,
                        hid: usb.hid,
                        remote_wakeup_signal: &remote_wakeup_signal,
                        hand,
                    })
                    .await
                },
            )
            .await
        },
    )
    .await;
}
