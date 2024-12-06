//! Common definitions for the Keyball keyboard firmware. Independent of the specific MCU used.
#![no_std]

pub mod keymap;

pub use keymap::KEYMAP;

use rktk_drivers_common::{keyscan::duplex_matrix::ScanDir, mouse::paw3395, usb::UsbDriverConfig};

pub const PAW3395_CONFIG: paw3395::config::Config = paw3395::config::Config {
    mode: paw3395::config::HP_MODE,
    lift_cutoff: paw3395::config::LiftCutoff::_2mm,
};

pub const USB_CONFIG: UsbDriverConfig = {
    let mut config = UsbDriverConfig::new(0xc0de, 0xcafe);

    config.manufacturer = Some("Yowkees/nazo6");
    config.product = Some("keyball");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;
    config.supports_remote_wakeup = true;

    config
};

// Left
//    [COL2ROW] [ROW2COL]
// COL 0 1 2    0 1 2 3
//
//     0 1 2    3 4 5 6
pub fn translate_key_position(dir: ScanDir, row: usize, col: usize) -> Option<(usize, usize)> {
    match dir {
        ScanDir::Col2Row => {
            if col == 3 {
                return None;
            }
            Some((row, col))
        }
        ScanDir::Row2Col => Some((row, col + 3)),
    }
}
