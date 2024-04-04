mod registers;
mod srom_liftoff;
mod srom_tracking;

use core::convert::Infallible;

use embassy_rp::{
    gpio::Output,
    spi::{Async, Instance, Spi},
};
use embassy_time::Timer;
use registers as reg;

#[derive(Default)]
pub struct BurstData {
    pub motion: bool,
    pub on_surface: bool,
    pub dx: i16,
    pub dy: i16,
    pub surface_quality: u8,
    pub raw_data_sum: u8,
    pub max_raw_data: u8,
    pub min_raw_data: u8,
    pub shutter: u16,
}

pub struct Pmw3360<'d, T: Instance> {
    spi: Spi<'d, T, Async>,
    cs_pin: Output<'d>,
    // reset_pin: RESET,
    // rw_flag is set if any writes or reads were performed
    rw_flag: bool,
}

impl<'d, T: Instance> Pmw3360<'d, T> {
    pub async fn new(spi: Spi<'d, T, Async>, cs_pin: Output<'d>) -> Self {
        let mut pmw3360 = Self {
            spi,
            cs_pin,
            rw_flag: true,
        };

        pmw3360.power_up().await.unwrap();

        pmw3360
    }

    pub async fn burst_read(&mut self) -> Result<BurstData, Infallible> {
        // TODO: propagate errors

        // Write any value to Motion_burst register
        // if any write occured before
        if self.rw_flag {
            self.write(reg::MOTION_BURST, 0x00).await.ok();
            self.rw_flag = false;
        }

        // Lower NCS
        self.cs_pin.set_low();
        // Send Motion_burst address
        self.spi
            .transfer_in_place(&mut [reg::MOTION_BURST])
            .await
            .ok();

        // tSRAD_MOTBR
        Timer::after_micros(35).await;

        // Read the 12 bytes of burst data
        let mut buf = [0u8; 12];
        for i in 0..buf.len() {
            let t_buf = &mut [0x00];
            match self.spi.transfer_in_place(t_buf).await {
                Ok(()) => buf[i] = *t_buf.first().unwrap(),
                Err(_) => buf[i] = 0,
            }
        }

        // Raise NCS
        self.cs_pin.set_high();
        // tBEXIT
        Timer::after_micros(1).await;

        //combine the register values
        let data = BurstData {
            motion: (buf[0] & 0x80) != 0,
            on_surface: (buf[0] & 0x08) == 0, // 0 if on surface / 1 if off surface
            dx: (buf[3] as i16) << 8 | (buf[2] as i16),
            dy: (buf[5] as i16) << 8 | (buf[4] as i16),
            surface_quality: buf[6],
            raw_data_sum: buf[7],
            max_raw_data: buf[8],
            min_raw_data: buf[9],
            shutter: (buf[11] as u16) << 8 | (buf[10] as u16),
        };

        Ok(data)
    }

    pub async fn set_cpi(&mut self, cpi: u16) -> Result<(), Infallible> {
        let val: u16;
        if cpi < 100 {
            val = 0
        } else if cpi > 12000 {
            val = 0x77
        } else {
            val = (cpi - 100) / 100;
        }
        self.write(reg::CONFIG_1, val as u8).await.ok();
        Ok(())
    }

    pub async fn get_cpi(&mut self) -> Result<u16, Infallible> {
        let val = self.read(reg::CONFIG_1).await.unwrap_or_default() as u16;
        Ok((val + 1) * 100)
    }

    pub async fn check_signature(&mut self) -> Result<bool, Infallible> {
        // TODO: propagate errors

        let srom = self.read(reg::SROM_ID).await.unwrap_or(0);
        let pid = self.read(reg::PRODUCT_ID).await.unwrap_or(0);
        let ipid = self.read(reg::INVERSE_PRODUCT_ID).await.unwrap_or(0);

        // signature for SROM 0x04
        Ok(srom == 0x04 && pid == 0x42 && ipid == 0xBD)
    }

    #[allow(dead_code)]
    pub async fn self_test(&mut self) -> Result<bool, Infallible> {
        self.write(reg::SROM_ENABLE, 0x15).await.ok();
        Timer::after_micros(10000).await;

        let u = self.read(reg::DATA_OUT_UPPER).await.unwrap_or(0); // should be 0xBE
        let l = self.read(reg::DATA_OUT_LOWER).await.unwrap_or(0); // should be 0xEF

        Ok(u == 0xBE && l == 0xEF)
    }

    async fn write(&mut self, address: u8, data: u8) -> Result<(), Infallible> {
        // TODO: propagate errors

        self.cs_pin.set_low();
        // tNCS-SCLK
        Timer::after_micros(1).await;

        // send adress of the register, with MSBit = 1 to indicate it's a write
        self.spi.transfer_in_place(&mut [address | 0x80]).await.ok();
        // send data
        self.spi.transfer_in_place(&mut [data]).await.ok();

        // tSCLK-NCS (write)
        Timer::after_micros(35).await;
        self.cs_pin.set_high();

        // tSWW/tSWR minus tSCLK-NCS (write)
        Timer::after_micros(145).await;

        self.rw_flag = true;

        Ok(())
    }

    async fn read(&mut self, address: u8) -> Result<u8, Infallible> {
        // TODO: propagate errors
        self.cs_pin.set_low();
        // tNCS-SCLK
        Timer::after_micros(1).await;

        // send adress of the register, with MSBit = 0 to indicate it's a read
        self.spi.transfer_in_place(&mut [address & 0x7f]).await.ok();

        // tSRAD
        Timer::after_micros(160).await;

        let mut ret = 0;
        let mut buf = [0x00];
        if let Ok(_) = self.spi.transfer_in_place(&mut buf).await {
            ret = *buf.first().unwrap();
        }

        // tSCLK-NCS (read)
        Timer::after_micros(1).await;
        self.cs_pin.set_high();

        //  tSRW/tSRR minus tSCLK-NCS
        Timer::after_micros(20).await;

        self.rw_flag = true;

        Ok(ret)
    }

    async fn power_up(&mut self) -> Result<(), Infallible> {
        // TODO: propagate errors
        // sensor reset not active
        // self.reset_pin.set_high().ok();

        // reset the spi bus on the sensor
        self.cs_pin.set_high();
        Timer::after_micros(50).await;
        self.cs_pin.set_low();
        Timer::after_micros(50).await;

        // Write to reset register
        self.write(reg::POWER_UP_RESET, 0x5A).await.ok();
        // 100 ms delay
        Timer::after_micros(100).await;

        // read registers 0x02 to 0x06 (and discard the data)
        self.read(reg::MOTION).await.ok();
        self.read(reg::DELTA_X_L).await.ok();
        self.read(reg::DELTA_X_H).await.ok();
        self.read(reg::DELTA_Y_L).await.ok();
        self.read(reg::DELTA_Y_H).await.ok();

        // upload the firmware
        self.upload_fw().await.ok();

        let is_valid_signature = self.check_signature().await.unwrap_or(false);

        // Write 0x00 (rest disable) to Config2 register for wired mouse or 0x20 for
        // wireless mouse design.
        self.write(reg::CONFIG_2, 0x00).await.ok();

        Timer::after_micros(100).await;

        if is_valid_signature {
            return Ok(());
        };

        Ok(())
    }

    async fn upload_fw(&mut self) -> Result<(), Infallible> {
        // TODO: propagate errors
        // Write 0 to Rest_En bit of Config2 register to disable Rest mode.
        self.write(reg::CONFIG_2, 0x00).await.ok();

        // write 0x1d in SROM_enable reg for initializing
        self.write(reg::SROM_ENABLE, 0x1d).await.ok();

        // wait for 10 ms
        Timer::after_micros(10000).await;

        // write 0x18 to SROM_enable to start SROM download
        self.write(reg::SROM_ENABLE, 0x18).await.ok();

        // lower NCS
        self.cs_pin.set_low();

        // first byte is address
        self.spi
            .transfer_in_place(&mut [reg::SROM_LOAD_BURST | 0x80])
            .await
            .ok();
        Timer::after_micros(15).await;

        // send the rest of the firmware
        for element in srom_tracking::FW.iter() {
            self.spi.transfer_in_place(&mut [*element]).await.ok();
            Timer::after_micros(15).await;
        }

        Timer::after_micros(2).await;
        self.cs_pin.set_high();
        Timer::after_micros(200).await;
        Ok(())
    }
}
