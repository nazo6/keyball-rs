use core::sync::atomic::AtomicBool;

use embassy_usb::class::hid::{HidReaderWriter, RequestHandler, State};
use embassy_usb::{Builder, Config, Handler, UsbDevice};
use usbd_hid::descriptor::{KeyboardReport, MouseReport, SerializedDescriptor};

use crate::device::usb::DeviceDriver;

pub mod device_handler;
pub mod request_handler;

pub struct Hid<'a> {
    pub keyboard: HidReaderWriter<'a, DeviceDriver<'a>, 1, 8>,
    pub mouse: HidReaderWriter<'a, DeviceDriver<'a>, 1, 8>,
}

pub struct UsbOpts<'a> {
    pub driver: DeviceDriver<'a>,
    pub config_descriptor: &'a mut [u8],
    pub bos_descriptor: &'a mut [u8],
    pub msos_descriptor: &'a mut [u8],
    pub control_buf: &'a mut [u8],
    pub kb_request_handler: &'a mut dyn RequestHandler,
    pub mouse_request_handler: &'a mut dyn RequestHandler,
    pub device_handler: &'a mut dyn Handler,
    pub state_kb: &'a mut State<'a>,
    pub state_mouse: &'a mut State<'a>,
}

pub struct UsbResource<'a> {
    pub device: UsbDevice<'a, DeviceDriver<'a>>,
    pub hid: Hid<'a>,
}

pub static SUSPENDED: AtomicBool = AtomicBool::new(false);

pub fn create_usb(opts: UsbOpts) -> UsbResource {
    // Create embassy-usb Config
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Yowkees/nazo6");
    config.product = Some("keyball");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;
    config.supports_remote_wakeup = true;

    let mut builder = Builder::new(
        opts.driver,
        config,
        opts.config_descriptor,
        opts.bos_descriptor,
        opts.msos_descriptor,
        opts.control_buf,
    );

    builder.handler(opts.device_handler);

    let keyboard_hid = {
        let config = embassy_usb::class::hid::Config {
            report_descriptor: KeyboardReport::desc(),
            request_handler: Some(opts.kb_request_handler),
            poll_ms: 10,
            max_packet_size: 64,
        };
        HidReaderWriter::<_, 1, 8>::new(&mut builder, opts.state_kb, config)
    };
    let mouse_hid = {
        let config = embassy_usb::class::hid::Config {
            report_descriptor: MouseReport::desc(),
            request_handler: Some(opts.mouse_request_handler),
            poll_ms: 4,
            max_packet_size: 64,
        };
        HidReaderWriter::<_, 1, 8>::new(&mut builder, opts.state_mouse, config)
    };

    // Build the builder.
    let usb = builder.build();

    UsbResource {
        device: usb,
        hid: Hid {
            keyboard: keyboard_hid,
            mouse: mouse_hid,
        },
    }
}
