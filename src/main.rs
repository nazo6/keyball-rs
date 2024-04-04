#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Flex, Level, Output, Pull};
use embassy_rp::peripherals::USB;
use embassy_rp::spi::Spi;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use embassy_usb::class::hid::State;
use keycodes::KC_NO;
use ssd1306::Ssd1306Display;
use usb_handler::{MyDeviceHandler, MyRequestHandler};
use usbd_hid::descriptor::KeyboardReport;

mod double_reset;
mod keycodes;
mod keymap;
mod pmw3360;
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

    // Display
    let mut i2c_config = embassy_rp::i2c::Config::default();
    i2c_config.frequency = 400_000;

    let i2c = embassy_rp::i2c::I2c::new_blocking(p.I2C1, p.PIN_3, p.PIN_2, i2c_config);
    let display = ssd1306::Ssd1306Display::new(i2c);
    *(DISPLAY.lock()).await = Some(display);

    // Ball
    let mut spi_config = embassy_rp::spi::Config::default();
    spi_config.frequency = 2_000_000;
    spi_config.polarity = embassy_rp::spi::Polarity::IdleHigh;
    spi_config.phase = embassy_rp::spi::Phase::CaptureOnSecondTransition;
    let spi = Spi::new(
        p.SPI0, p.PIN_22, p.PIN_23, p.PIN_20, p.DMA_CH0, p.DMA_CH1, spi_config,
    );
    let mut pmw3360 = pmw3360::Pmw3360::new(spi, Output::new(p.PIN_21, Level::High)).await;
    let pmw_fut = async move {
        loop {
            let data = pmw3360.burst_read().await.unwrap();
            let mut str = heapless::String::<100>::new();
            write!(
                &mut str,
                "dx: {}, dy: {},\nquality: {}",
                data.dx, data.dy, data.surface_quality
            )
            .unwrap();
            // DISPLAY.lock().await.as_mut().unwrap().draw_text(&str);
            Timer::after_millis(50).await;
        }
    };

    // USB Keyboard

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
        Flex::new(p.PIN_4),
        Flex::new(p.PIN_5),
        Flex::new(p.PIN_6),
        Flex::new(p.PIN_7),
        Flex::new(p.PIN_8),
    ];
    let mut cols = [
        Flex::new(p.PIN_26),
        Flex::new(p.PIN_27),
        Flex::new(p.PIN_28),
        Flex::new(p.PIN_29),
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
            let mut keys = heapless::Vec::<(usize, usize), 100>::new();

            for row in rows.iter_mut() {
                row.set_as_output();
                row.set_low();
            }
            for col in cols.iter_mut() {
                col.set_as_input();
                col.set_pull(Pull::Down);
            }

            for (i, row) in rows.iter_mut().enumerate() {
                row.set_high();
                row.wait_for_high().await;

                for (j, col) in cols.iter_mut().enumerate() {
                    if col.is_high() {
                        keys.push((i, j)).unwrap();
                    }
                }

                row.set_low();
                row.wait_for_low().await;
            }

            for row in rows.iter_mut() {
                row.set_as_input();
                row.set_pull(Pull::Down);
            }
            for col in cols.iter_mut() {
                col.set_as_output();
                col.set_low();
            }

            for (j, col) in cols.iter_mut().enumerate() {
                col.set_high();
                col.wait_for_high().await;

                for (i, row) in rows.iter_mut().enumerate() {
                    if row.is_high() {
                        keys.push((i, j + 3)).unwrap();
                    }
                }

                col.set_low();
                col.wait_for_low().await;
            }

            // DISPLAY.lock().await.as_mut().unwrap().draw_text(&str);

            let mut keycodes = [0; 6];
            let mut idx = 0;
            for (row, col) in keys.iter() {
                if idx >= keycodes.len() {
                    break;
                }
                let kc = keymap::KEYMAP[*row][*col + 7];
                if kc == KC_NO {
                    continue;
                }
                keycodes[idx] = kc;
                idx += 1;
            }

            let report = KeyboardReport {
                keycodes,
                leds: 0,
                modifier: 0,
                reserved: 0,
            };
            writer.write_serialize(&report).await;

            Timer::after_millis(10).await;
        }
    };

    let out_fut = async {
        reader.run(false, &request_handler).await;
    };

    let usb_fut = usb.run();

    // Run everything concurrently.
    // If we had made everything `'static` above instead, we could do this using separate tasks instead.
    join(join(usb_fut, join(in_fut, out_fut)), pmw_fut).await;
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    DISPLAY
        .try_lock()
        .unwrap()
        .as_mut()
        .unwrap()
        .draw_text("Panic!");

    loop {}
}
