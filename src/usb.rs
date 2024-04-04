use defmt_rtt as _;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
use embassy_usb::class::hid::{HidReaderWriter, RequestHandler, State};
use embassy_usb::{Builder, Config, Handler, UsbDevice};
use usbd_hid::descriptor::{KeyboardReport, MouseReport, SerializedDescriptor};

pub struct UsbOpts<'d, RH: RequestHandler, DH: Handler> {
    pub driver: Driver<'d, USB>,
    pub config_descriptor: &'d mut [u8],
    pub bos_descriptor: &'d mut [u8],
    pub msos_descriptor: &'d mut [u8],
    pub control_buf: &'d mut [u8],
    pub request_handler: &'d RH,
    pub device_handler: &'d mut DH,
    pub state_kb: &'d mut State<'d>,
    pub state_mouse: &'d mut State<'d>,
}

pub struct UsbRet<'d> {
    pub device: UsbDevice<'d, Driver<'d, USB>>,
    pub keyboard_hid: HidReaderWriter<'d, Driver<'d, USB>, 1, 8>,
    pub mouse_hid: HidReaderWriter<'d, Driver<'d, USB>, 1, 8>,
}

pub fn create_usb<RH: RequestHandler, DH: Handler>(opts: UsbOpts<RH, DH>) -> UsbRet {
    // Create embassy-usb Config
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Yowkees/nazo6");
    config.product = Some("keyball");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

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
            request_handler: Some(opts.request_handler),
            poll_ms: 60,
            max_packet_size: 64,
        };
        HidReaderWriter::<_, 1, 8>::new(&mut builder, opts.state_kb, config)
    };
    let mouse_hid = {
        let config = embassy_usb::class::hid::Config {
            report_descriptor: MouseReport::desc(),
            request_handler: Some(opts.request_handler),
            poll_ms: 60,
            max_packet_size: 64,
        };
        HidReaderWriter::<_, 1, 8>::new(&mut builder, opts.state_mouse, config)
    };

    // Build the builder.
    let usb = builder.build();

    UsbRet {
        device: usb,
        keyboard_hid,
        mouse_hid,
    }
}
