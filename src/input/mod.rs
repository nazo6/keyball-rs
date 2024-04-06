use core::fmt::Write as _;

use embassy_futures::join::join;
use embassy_rp::{
    bind_interrupts,
    peripherals::*,
    pio::{InterruptHandler, Pio},
    usb::Driver,
};
use embassy_time::Timer;
use embassy_usb::class::hid::HidReaderWriter;

use crate::DISPLAY;

mod ball;
mod keyboard;
mod led;
mod split;

pub struct InputPeripherals {
    pub keyboard: KeyboardPeripherals,
    pub ball: BallPeripherals,
    pub split: SplitInputPeripherals,
    pub led: LedPeripherals,
}

pub struct KeyboardPeripherals {
    pub row_0: PIN_4,
    pub row_1: PIN_5,
    pub row_2: PIN_6,
    pub row_3: PIN_7,
    pub row_4: PIN_8,
    pub col_0: PIN_26,
    pub col_1: PIN_27,
    pub col_2: PIN_28,
    pub col_3: PIN_29,
}

pub struct BallPeripherals {
    pub spi: SPI0,
    pub spi_clk: PIN_22,
    pub spi_mosi: PIN_23,
    pub spi_miso: PIN_20,
    pub spi_dma_ch0: DMA_CH0,
    pub spi_dma_ch1: DMA_CH1,
    pub ncs: PIN_21,
}

pub struct SplitInputPeripherals {
    pub pio: PIO0,
    pub data_pin: PIN_1,
}

pub struct LedPeripherals {
    pub pio: PIO1,
    pub led_pin: PIN_0,
    pub dma: DMA_CH2,
}

pub struct Hid<'a> {
    pub keyboard: HidReaderWriter<'a, Driver<'a, USB>, 1, 8>,
    pub mouse: HidReaderWriter<'a, Driver<'a, USB>, 1, 8>,
}

/// Starts the input task.
/// If hid is Some, this is master side, and report will be sent to the USB device.
/// If hid is None, this is slave side, and report will be sent to the master.
pub async fn start(peripherals: InputPeripherals, hid: Option<Hid<'_>>) {
    let hid = hid.unwrap();
    let (kb_reader, mut kb_writer) = hid.keyboard.split();
    let (mouse_reader, mut mouse_writer) = hid.mouse.split();

    let mut ball = ball::Ball::init(peripherals.ball).await;
    let mut keyboard = keyboard::Keyboard::new(peripherals.keyboard);

    let poll_fut = async {
        let mut empty_kb_sent = false;
        loop {
            let (ball, keyboard) = join(ball.as_mut().unwrap().read(), keyboard.read()).await;

            join(
                async {
                    if let Some(kb_report) = keyboard {
                        kb_writer.write_serialize(&kb_report).await;
                        empty_kb_sent = false;
                    } else if !empty_kb_sent {
                        kb_writer
                            .write_serialize(&usbd_hid::descriptor::KeyboardReport {
                                keycodes: [0; 6],
                                leds: 0,
                                modifier: 0,
                                reserved: 0,
                            })
                            .await;
                        empty_kb_sent = true;
                    }
                },
                async {
                    if let Some(mouse_report) = ball {
                        mouse_writer.write_serialize(&mouse_report).await;
                    }
                },
            )
            .await;

            Timer::after_millis(10).await;
        }
    };
    join(
        poll_fut,
        join(split::start(peripherals.split), led::start(peripherals.led)),
    )
    .await;
}
