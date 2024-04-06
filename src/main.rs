#![no_std]
#![no_main]

use core::fmt::Write as _;
use core::panic::PanicInfo;

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_usb::class::hid::State;
use input::{Hid, InputPeripherals};
use oled::Oled;
use usb::UsbOpts;
use usb_handler::{UsbDeviceHandler, UsbRequestHandler};

mod double_reset;
mod input;
mod keycodes;
mod keymap;
mod oled;
mod usb;
mod usb_handler;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

type DisplayType = Mutex<ThreadModeRawMutex, Option<Oled<'static>>>;
static DISPLAY: DisplayType = Mutex::new(None);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    unsafe { double_reset::check_double_tap_bootloader(500).await };

    // Display
    let mut display = oled::Oled::new(oled::DisplayPeripherals {
        i2c: p.I2C1,
        scl: p.PIN_3,
        sda: p.PIN_2,
    });

    display.draw_text("Hello world!");

    *(DISPLAY.lock()).await = Some(display);

    // Usb keyboard and mouse
    let opts = UsbOpts {
        driver: Driver::new(p.USB, Irqs),
        config_descriptor: &mut [0; 256],
        bos_descriptor: &mut [0; 256],
        msos_descriptor: &mut [0; 256],
        control_buf: &mut [0; 64],
        request_handler: &UsbRequestHandler {},
        device_handler: &mut UsbDeviceHandler::new(),
        state_kb: &mut State::new(),
        state_mouse: &mut State::new(),
    };
    let mut usb = usb::create_usb(opts);

    let usb_fut = async { usb.device.run().await };
    let input_fut = input::start(
        InputPeripherals {
            keyboard: input::KeyboardPeripherals {
                row_0: p.PIN_4,
                row_1: p.PIN_5,
                row_2: p.PIN_6,
                row_3: p.PIN_7,
                row_4: p.PIN_8,
                col_0: p.PIN_26,
                col_1: p.PIN_27,
                col_2: p.PIN_28,
                col_3: p.PIN_29,
            },
            ball: input::BallPeripherals {
                spi: p.SPI0,
                spi_clk: p.PIN_22,
                spi_mosi: p.PIN_23,
                spi_miso: p.PIN_20,
                spi_dma_ch0: p.DMA_CH0,
                spi_dma_ch1: p.DMA_CH1,
                ncs: p.PIN_21,
            },
            split: input::SplitInputPeripherals {
                pio: p.PIO0,
                data_pin: p.PIN_1,
            },
            led: input::LedPeripherals {
                pio: p.PIO1,
                led_pin: p.PIN_0,
                dma: p.DMA_CH2,
            },
        },
        Some(Hid {
            keyboard: usb.keyboard_hid,
            mouse: usb.mouse_hid,
        }),
    );

    join(usb_fut, input_fut).await;
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
