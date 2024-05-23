use crate::device::adc::{create_adc, temp_sensor_channel};

use super::TemperaturePeripherals;

/// Task to read and display temperature.
/// This is for debugging purposes.
pub async fn start(p: TemperaturePeripherals) {
    let mut adc = create_adc(p.adc);
    let mut tch = temp_sensor_channel(p.temp_sensor);
    loop {
        if let Ok(temp) = adc.read(&mut tch).await {
            crate::print!("TEMP: {}", convert_to_celsius(temp));
        }
        embassy_time::Timer::after_secs(1).await;
    }
}

// https://github.com/embassy-rs/embassy/blob/main/examples/rp/src/bin/adc.rs
fn convert_to_celsius(raw_temp: u16) -> f32 {
    // According to chapter 4.9.5. Temperature Sensor in RP2040 datasheet
    let temp = 27.0 - (raw_temp as f32 * 3.3 / 4096.0 - 0.706) / 0.001721;
    let sign = if temp < 0.0 { -1.0 } else { 1.0 };
    let rounded_temp_x10: i16 = ((temp * 10.0) + 0.5 * sign) as i16;
    (rounded_temp_x10 as f32) / 10.0
}
