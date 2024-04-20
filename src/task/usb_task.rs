use embassy_futures::select::{select, Either};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_usb::UsbDevice;

use crate::device::usb::DeviceDriver;

pub type RemoteWakeupSignal = embassy_sync::signal::Signal<CriticalSectionRawMutex, ()>;

pub struct UsbTaskResource<'a> {
    pub device: UsbDevice<'a, DeviceDriver<'a>>,
    pub signal: &'a RemoteWakeupSignal,
}
pub async fn start(UsbTaskResource { mut device, signal }: UsbTaskResource<'_>) {
    loop {
        device.run_until_suspend().await;
        crate::DISPLAY.set_message("suspend").await;
        match select(device.wait_resume(), signal.wait()).await {
            Either::First(_) => {
                crate::DISPLAY.set_message("resume").await;
            }
            Either::Second(_) => {
                let _ = device.remote_wakeup().await;
            }
        }
    }
}
