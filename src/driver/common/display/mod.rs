/// SSD1306 OLED module
use core::fmt::Write;
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

use crate::device::{
    i2c_display::{create_i2c, I2C},
    peripherals::DisplayPeripherals,
};

pub struct Oled<'a> {
    display:
        Ssd1306<I2CInterface<I2C<'a>>, DisplaySize128x32, BufferedGraphicsMode<DisplaySize128x32>>,
}

impl<'a> Oled<'a> {
    pub fn new(p: DisplayPeripherals) -> Self {
        let interface = I2CDisplayInterface::new(create_i2c(p, 400_000));

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
