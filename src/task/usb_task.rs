use embassy_futures::select::{select, Either};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_usb::UsbDevice;

use crate::device::usb::{remote_wakeup, DeviceDriver};

pub type RemoteWakeupSignal = embassy_sync::signal::Signal<CriticalSectionRawMutex, ()>;

pub struct UsbTaskResource<'a> {
    pub device: UsbDevice<'a, DeviceDriver<'a>>,
    pub signal: &'a RemoteWakeupSignal,
}
pub async fn start(UsbTaskResource { mut device, signal }: UsbTaskResource<'_>) {
    loop {
        device.run_until_suspend().await;
        match select(device.wait_resume(), signal.wait()).await {
            Either::First(_) => {}
            Either::Second(_) => {
                // embassy-rpがremote wakeupをサポートしてないのでデバイス固有実装
                remote_wakeup(&mut device).await;
            }
        }
    }
}
