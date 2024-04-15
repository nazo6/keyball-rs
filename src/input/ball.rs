use embassy_rp::{
    gpio::{Level, Output},
    peripherals::SPI0,
    spi::Spi,
};
use embassy_time::Timer;
use usbd_hid::descriptor::MouseReport;

use super::BallPeripherals;

mod pmw3360;

// high-level interface to interact with the PMW3360 sensor
pub struct Ball<'d> {
    driver: pmw3360::Pmw3360<'d, SPI0>,
}

impl<'d> Ball<'d> {
    /// Initializes the ball sensor and returns ball instance.
    /// If no sensor is found, returns None.
    pub async fn init(p: BallPeripherals) -> Result<Self, pmw3360::Pmw3360Error> {
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
        let mut pmw3360 = pmw3360::Pmw3360::new(spi, Output::new(p.ncs, Level::High)).await?;

        Timer::after_millis(50).await;

        let _ = pmw3360.set_cpi(300).await;

        Ok(Self { driver: pmw3360 })
    }

    /// Reads the sensor data.
    pub async fn read(&mut self) -> Result<Option<MouseReport>, pmw3360::Pmw3360Error> {
        let data = self.driver.burst_read().await?;

        if data.dx == 0 && data.dy == 0 {
            return Ok(None);
        }

        Ok(Some(MouseReport {
            buttons: 0,
            x: data.dy as i8,
            y: data.dx as i8,
            wheel: 0,
            pan: 0,
        }))
    }
}
