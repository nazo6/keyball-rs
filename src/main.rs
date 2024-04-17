#![no_std]
#![no_main]

use core::fmt::Write as _;
use core::panic::PanicInfo;

use defmt_rtt as _;
use device::peripherals::init_peripherals;
use display::DISPLAY;
use embassy_executor::Spawner;
use task::TaskPeripherals;

mod constant;
mod device;
mod display;
mod driver;
mod keyboard;
mod task;
mod usb;
mod utils;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let peripherals = init_peripherals();

    driver::double_tap::check_double_tap(500).await;

    DISPLAY.init(peripherals.display).await;

    task::start(TaskPeripherals {
        ball: peripherals.ball,
        keyboard: peripherals.keyboard,
        split: peripherals.split,
        led: peripherals.led,
        usb: peripherals.usb,
    })
    .await;
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut str = heapless::String::<1000>::new();
    let loc = info.location().unwrap();
    write!(&mut str, "P:\n{}\n{}\n", loc.file(), loc.line()).unwrap();

    DISPLAY.try_draw_text(&str);

    loop {}
}
