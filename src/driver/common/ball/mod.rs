use crate::device::{
    gpio::{Level, Output},
    spi_ball::create_ball_spi,
};
use embassy_time::Timer;

use crate::device::peripherals::BallPeripherals;

mod pmw3360;

// high-level interface to interact with the PMW3360 sensor
pub struct Ball<'d> {
    driver: pmw3360::Pmw3360<'d>,
}

impl<'d> Ball<'d> {
    /// Initializes the ball sensor and returns ball instance.
    /// If no sensor is found, returns None.
    pub async fn init(p: BallPeripherals) -> Result<Self, pmw3360::Pmw3360Error> {
        let spi = create_ball_spi(p.spi);
        let mut pmw3360 = pmw3360::Pmw3360::new(spi, Output::new(p.ncs, Level::High)).await?;

        Timer::after_millis(50).await;

        let _ = pmw3360.set_cpi(300).await;

        Ok(Self { driver: pmw3360 })
    }

    /// Reads the sensor data.
    pub async fn read(&mut self) -> Result<Option<(i8, i8)>, pmw3360::Pmw3360Error> {
        let data = self.driver.burst_read().await?;

        if data.dx == 0 && data.dy == 0 {
            return Ok(None);
        }

        Ok(Some((data.dx as i8, data.dy as i8)))
    }
}
