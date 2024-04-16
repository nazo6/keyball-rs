/// SSD1306 OLED module
use core::fmt::Write;
use embassy_rp::{
    i2c::{Blocking, I2c},
    peripherals::{I2C1, PIN_2, PIN_3},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

pub struct Oled<'a> {
    display: Ssd1306<
        I2CInterface<I2c<'a, I2C1, Blocking>>,
        DisplaySize128x32,
        BufferedGraphicsMode<DisplaySize128x32>,
    >,
}

pub struct DisplayPeripherals {
    pub i2c: I2C1,
    pub scl: PIN_3,
    pub sda: PIN_2,
}

impl<'a> Oled<'a> {
    pub fn new(p: DisplayPeripherals) -> Self {
        let mut i2c_config = embassy_rp::i2c::Config::default();
        i2c_config.frequency = 400_000;

        let i2c = embassy_rp::i2c::I2c::new_blocking(p.i2c, p.scl, p.sda, i2c_config);

        let interface = I2CDisplayInterface::new(i2c);
        let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
        display.init().unwrap();

        Self { display }
    }

    pub fn draw_text(&mut self, text: &str) {
        self.display.clear_buffer();

        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();

        Text::with_baseline(text, Point::zero(), text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();

        self.display.flush().unwrap();
    }

    pub fn draw_number(&mut self, number: u32) {
        self.display.clear_buffer();
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();
        let mut str = heapless::String::<100>::new();
        write!(str, "{}", number).unwrap();
        Text::with_baseline(&str, Point::zero(), text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
        self.display.flush().unwrap();
    }
}
