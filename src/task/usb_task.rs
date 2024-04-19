use embassy_futures::select::{select, Either};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_usb::UsbDevice;

use crate::device::usb::DeviceDriver;

pub type RemoteWakeupSignal = embassy_sync::signal::Signal<CriticalSectionRawMutex, ()>;

pub async fn start<'a>(mut device: UsbDevice<'a, DeviceDriver<'a>>, signal: &RemoteWakeupSignal) {
    loop {
        device.run_until_suspend().await;
        match select(device.wait_resume(), signal.wait()).await {
            Either::First(_) => (),
            Either::Second(_) => {
                let _ = device.remote_wakeup().await;
            }
        }
    }
}
