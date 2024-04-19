use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

use crate::device::{
    i2c_display::{create_i2c, I2C},
    peripherals::DisplayPeripherals,
};

const TEXT_STYLE: MonoTextStyle<'static, BinaryColor> = MonoTextStyleBuilder::new()
    .font(&FONT_6X10)
    .text_color(BinaryColor::On)
    .background_color(BinaryColor::Off)
    .build();

/// SSD1306 OLED module
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

    pub const fn calculate_point(col: i32, row: i32) -> Point {
        Point::new((col - 1) * 6, (row - 1) * 10)
    }

    pub async fn clear(&mut self) {
        self.display.clear_buffer();
        self.display.flush_async().await.unwrap();
    }

    pub async fn draw_text(&mut self, text: &str) {
        self.display.clear_buffer();

        Text::with_baseline(text, Point::new(0, 0), TEXT_STYLE, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();

        self.display.flush_async().await.unwrap();
    }

    pub async fn update_text(&mut self, text: &str, point: Point) {
        Text::with_baseline(text, point, TEXT_STYLE, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();

        self.display.flush_async().await.unwrap();
    }

    pub fn draw_text_blocking(&mut self, text: &str) {
        self.display.clear_buffer();

        Text::with_baseline(text, Point::zero(), TEXT_STYLE, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();

        self.display.flush().unwrap();
    }

    pub fn update_text_blocking(&mut self, text: &str, point: Point) {
        Text::with_baseline(text, point, TEXT_STYLE, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();
        self.display.flush().unwrap();
    }
}
