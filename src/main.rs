#![no_std]
#![no_main]

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
use input::InputPeripherals;
use ssd1306::Ssd1306Display;
use usb::UsbOpts;
use usb_handler::{UsbDeviceHandler, UsbRequestHandler};

mod double_reset;
mod input;
mod keycodes;
mod keymap;
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

    // USB Keyboard

    // Create the driver, from the HAL.
    let driver = Driver::new(p.USB, Irqs);

    let opts = UsbOpts {
        driver,
        config_descriptor: &mut [0; 256],
        bos_descriptor: &mut [0; 256],
        msos_descriptor: &mut [0; 256],
        control_buf: &mut [0; 64],
        request_handler: &UsbRequestHandler {},
        device_handler: &mut UsbDeviceHandler::new(),
        state_kb: &mut State::new(),
        state_mouse: &mut State::new(),
    };

    let mut usb = usb::start_usb(opts);

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
        },
        usb.keyboard_hid,
        usb.mouse_hid,
    );

    join(usb_fut, input_fut).await;
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
