use core::fmt::Write as _;

use embassy_futures::join::join;
use embassy_rp::{peripherals::*, usb::Driver};
use embassy_time::Timer;
use embassy_usb::class::hid::HidReaderWriter;

use crate::DISPLAY;

mod ball;
mod keyboard;

pub struct InputPeripherals {
    pub keyboard: KeyboardPeripherals,
    pub ball: BallPeripherals,
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

pub async fn start<'a>(
    peripherals: InputPeripherals,
    hid_keyboard: HidReaderWriter<'a, Driver<'a, USB>, 1, 8>,
    hid_mouse: HidReaderWriter<'a, Driver<'a, USB>, 1, 8>,
) {
    let (kb_reader, mut kb_writer) = hid_keyboard.split();
    let (mouse_reader, mut mouse_writer) = hid_mouse.split();

    let mut ball = ball::Ball::init(peripherals.ball).await;
    let mut keyboard = keyboard::Keyboard::new(peripherals.keyboard);
    loop {
        let (ball, keyboard) = join(ball.read(), keyboard.read()).await;

        let mut str = heapless::String::<100>::new();
        write!(&mut str, "dx: {}, dy: {}", ball.x, ball.y).unwrap();
        DISPLAY.lock().await.as_mut().unwrap().draw_text(&str);

        join(
            kb_writer.write_serialize(&keyboard),
            mouse_writer.write_serialize(&ball),
        )
        .await;

        Timer::after_millis(50).await;
    }
}
