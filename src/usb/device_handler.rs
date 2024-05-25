use core::sync::atomic::{AtomicBool, Ordering};

use embassy_usb::Handler;

use super::SUSPENDED;

// use crate::utils::print_sync;

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

// 参考: https://www.itf.co.jp/tech/road-to-usb-master/usb-status
impl Handler for UsbDeviceHandler {
    fn enabled(&mut self, enabled: bool) {
        self.configured.store(false, Ordering::Relaxed);
        SUSPENDED.store(false, Ordering::Release);
    }

    fn reset(&mut self) {
        self.configured.store(false, Ordering::Relaxed);
        // print_sync!("Bus reset, the Vbus current limit is 100mA");
    }

    fn addressed(&mut self, _addr: u8) {
        self.configured.store(false, Ordering::Relaxed);
        // print_sync!("USB address set to: {}", addr);
    }

    fn configured(&mut self, configured: bool) {
        self.configured.store(configured, Ordering::Relaxed);
    }

    fn suspended(&mut self, suspended: bool) {
        SUSPENDED.store(suspended, Ordering::Release);
    }
}
