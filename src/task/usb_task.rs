use embassy_usb::UsbDevice;

use crate::device::usb::DeviceDriver;

pub async fn start<'a>(mut device: UsbDevice<'a, DeviceDriver<'a>>) {
    device.run().await;
}
