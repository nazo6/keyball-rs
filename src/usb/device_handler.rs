use core::sync::atomic::{AtomicBool, Ordering};

use embassy_usb::Handler;

use super::SUSPENDED;

use crate::utils::print_sync;

pub struct UsbDeviceHandler {
    configured: AtomicBool,
}

impl UsbDeviceHandler {
    pub fn new() -> Self {
        UsbDeviceHandler {
            configured: AtomicBool::new(false),
        }
    }
}

impl Handler for UsbDeviceHandler {
    fn enabled(&mut self, enabled: bool) {
        self.configured.store(false, Ordering::Relaxed);
        if enabled {
            // print_sync!("Device enabled");
        } else {
            // print_sync!("Device disabled");
        }
    }

    fn reset(&mut self) {
        self.configured.store(false, Ordering::Relaxed);
        // print_sync!("Bus reset, the Vbus current limit is 100mA");
    }

    fn addressed(&mut self, addr: u8) {
        self.configured.store(false, Ordering::Relaxed);
        // print_sync!("USB address set to: {}", addr);
    }

    fn configured(&mut self, configured: bool) {
        self.configured.store(configured, Ordering::Relaxed);
        if configured {
            // print_sync!(
            //     "Device configured, it may now draw up to the configured current limit from Vbus."
            // )
        } else {
            // print_sync!("Device is no longer configured, the Vbus current limit is 100mA.");
        }
    }

    fn suspended(&mut self, suspended: bool) {
        if suspended {
            print_sync!("Suspended");
        } else {
            print_sync!("Unsuspended");
        }
        SUSPENDED.store(suspended, Ordering::Relaxed);
    }
}
