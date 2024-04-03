use embassy_rp::{
    i2c::{Blocking, I2c},
    peripherals::I2C1,
};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

pub struct Ssd1306Display<'a> {
    display: Ssd1306<
        I2CInterface<I2c<'a, I2C1, Blocking>>,
        DisplaySize128x32,
        BufferedGraphicsMode<DisplaySize128x32>,
    >,
}

impl<'a> Ssd1306Display<'a> {
    pub fn new(i2c: I2c<'a, I2C1, Blocking>) -> Self {
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
}
