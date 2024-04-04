use embassy_rp::{
    gpio::{Level, Output},
    peripherals::SPI0,
    spi::Spi,
};
use usbd_hid::descriptor::MouseReport;

use super::BallPeripherals;

mod pmw3360;

pub struct Ball<'d> {
    driver: pmw3360::Pmw3360<'d, SPI0>,
}

impl<'d> Ball<'d> {
    /// Initializes the ball sensor.
    pub async fn init(p: BallPeripherals) -> Self {
        let mut spi_config = embassy_rp::spi::Config::default();
        spi_config.frequency = 2_000_000;
        spi_config.polarity = embassy_rp::spi::Polarity::IdleHigh;
        spi_config.phase = embassy_rp::spi::Phase::CaptureOnSecondTransition;
        let spi = Spi::new(
            p.spi,
            p.spi_clk,
            p.spi_mosi,
            p.spi_miso,
            p.spi_dma_ch0,
            p.spi_dma_ch1,
            spi_config,
        );
        let mut pmw3360 = pmw3360::Pmw3360::new(spi, Output::new(p.ncs, Level::High)).await;

        pmw3360.set_cpi(600).await;

        Self { driver: pmw3360 }
    }

    /// Reads the sensor data.
    pub async fn read(&mut self) -> MouseReport {
        let data = self.driver.burst_read().await.unwrap();

        MouseReport {
            buttons: 0,
            x: data.dx as i8,
            y: data.dy as i8,
            wheel: 0,
            pan: 0,
        }
    }
}
