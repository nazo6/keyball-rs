#![no_std]
#![no_main]

use core::fmt::Write as _;
use core::panic::PanicInfo;

use defmt_rtt as _;
use device::peripherals::init_peripherals;
use device::usb::create_usb_driver;
use display::DISPLAY;
use embassy_executor::Spawner;
use embassy_usb::class::hid::State;
use usb::device_handler::UsbDeviceHandler;
use usb::request_handler::UsbRequestHandler;
use usb::UsbOpts;

mod constant;
mod device;
mod display;
mod double_reset;
mod driver;
mod keyconfig;
mod task;
mod usb;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripherals = init_peripherals();

    unsafe { double_reset::check_double_tap_bootloader(500).await };

    display::init_display(peripherals.display).await;

    let mut device_handler = UsbDeviceHandler::new();

    // Usb keyboard and mouse
    let opts = UsbOpts {
        driver: create_usb_driver(peripherals.usb),
        config_descriptor: &mut [0; 256],
        bos_descriptor: &mut [0; 256],
        msos_descriptor: &mut [0; 256],
        control_buf: &mut [0; 64],
        request_handler: &UsbRequestHandler {},
        device_handler: &mut device_handler,
        state_kb: &mut State::new(),
        state_mouse: &mut State::new(),
    };
    let usb = usb::create_usb(opts);

    task::start(task::TaskResource {
        usb,
        peripherals: task::TaskPeripherals {
            keyboard: peripherals.keyboard,
            ball: peripherals.ball,
            split: peripherals.split,
            led: peripherals.led,
        },
    })
    .await;
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut str = heapless::String::<1000>::new();

    let loc = info.location().unwrap();
    write!(&mut str, "P:\n{}\n{}\n", loc.file(), loc.line()).unwrap();

    DISPLAY
        .try_lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .draw_text(&str);

    loop {}
}
