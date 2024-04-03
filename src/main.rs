#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pin, Pull};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use embassy_usb::class::hid::State;
use ssd1306::Ssd1306Display;
use usb_handler::{MyDeviceHandler, MyRequestHandler};
use usbd_hid::descriptor::KeyboardReport;

mod double_reset;
mod ssd1306;
mod usb;
mod usb_handler;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

type DisplayType = Mutex<ThreadModeRawMutex, Option<Ssd1306Display<'static>>>;
static DISPLAY: DisplayType = Mutex::new(None);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    unsafe { double_reset::check_double_tap_bootloader(500).await };

    let mut i2c_config = embassy_rp::i2c::Config::default();
    i2c_config.frequency = 400_000;

    let i2c = embassy_rp::i2c::I2c::new_blocking(p.I2C1, p.PIN_3, p.PIN_2, i2c_config);
    let mut display = ssd1306::Ssd1306Display::new(i2c);
    *(DISPLAY.lock()).await = Some(display);

    // DISPLAY
    //     .lock()
    //     .await
    //     .as_mut()
    //     .unwrap()
    //     .draw_text("Hello from rust");

    // Create the driver, from the HAL.
    let driver = Driver::new(p.USB, Irqs);

    // ROW 0: GP4
    //     1: GP5
    //     2: GP6
    //     3: GP7
    //     4: GP8
    //
    // COL 0: GP29
    //     1: GP28
    //     2: GP27
    //     3: GP26

    let mut rows = [
        p.PIN_4.degrade(),
        p.PIN_5.degrade(),
        p.PIN_6.degrade(),
        p.PIN_7.degrade(),
        p.PIN_8.degrade(),
    ];
    let mut cols = [
        p.PIN_29.degrade(),
        p.PIN_28.degrade(),
        p.PIN_27.degrade(),
        p.PIN_26.degrade(),
    ];

    // Create embassy-usb DeviceBuilder using the driver and config.
    // It needs some buffers for building the descriptors.
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    // You can also add a Microsoft OS descriptor.
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; 64];

    let request_handler = MyRequestHandler {};
    let mut device_handler = MyDeviceHandler::new();

    let mut state = State::new();

    let (mut usb, hid) = {
        usb::start_usb(
            driver,
            &mut config_descriptor,
            &mut bos_descriptor,
            &mut msos_descriptor,
            &mut control_buf,
            &request_handler,
            &mut device_handler,
            &mut state,
        )
    };

    let (reader, mut writer) = hid.split();

    let in_fut = async {
        loop {
            let mut str = heapless::String::<100>::new();

            let rows_outputs = rows.iter_mut().map(|pin| Output::new(pin, Level::Low));
            for (i, mut row) in rows_outputs.enumerate() {
                row.set_high();
                let cols_inputs = cols.iter_mut().map(|pin| Input::new(pin, Pull::Down));
                for (j, col) in cols_inputs.enumerate() {
                    if col.is_high() {
                        write!(&mut str, "{},{} & ", i, j).unwrap();
                    }
                }
                row.set_low();
            }

            let cols_outputs = cols.iter_mut().map(|pin| Output::new(pin, Level::Low));
            for (j, mut col) in cols_outputs.enumerate() {
                col.set_high();
                let rows_inputs = rows.iter_mut().map(|pin| Input::new(pin, Pull::Down));
                for (i, row) in rows_inputs.enumerate() {
                    if row.is_high() {
                        write!(&mut str, "{},{} & ", i, j).unwrap();
                    }
                }
                col.set_low();
            }

            DISPLAY.lock().await.as_mut().unwrap().draw_text(&str);

            Timer::after_millis(50).await;

            // let report = KeyboardReport {
            //     keycodes: [4, 0, 0, 0, 0, 0],
            //     leds: 0,
            //     modifier: 0,
            //     reserved: 0,
            // };
            // // Send the report.
            // match writer.write_serialize(&report).await {
            //     Ok(()) => {}
            //     Err(e) => warn!("Failed to send report: {:?}", e),
            // };
        }
    };

    let out_fut = async {
        reader.run(false, &request_handler).await;
    };

    let usb_fut = usb.run();

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(usb_fut, join(in_fut, out_fut)).await;
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    DISPLAY
        .try_lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .draw_text("Panic!");

    loop {}
}
