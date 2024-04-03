use defmt_rtt as _;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
use embassy_usb::class::hid::{HidReaderWriter, RequestHandler, State};
use embassy_usb::{Builder, Config, Handler, UsbDevice};
use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};

pub fn start_usb<'d>(
    driver: Driver<'d, USB>,
    config_descriptor: &'d mut [u8],
    bos_descriptor: &'d mut [u8],
    msos_descriptor: &'d mut [u8],
    control_buf: &'d mut [u8],
    request_handler: &'d dyn RequestHandler,
    device_handler: &'d mut dyn Handler,
    state: &'d mut State<'d>,
) -> (
    UsbDevice<'d, Driver<'d, USB>>,
    HidReaderWriter<'d, Driver<'d, USB>, 1, 8>,
) {
    // Create embassy-usb Config
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("HID keyboard example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    let mut builder = Builder::new(
        driver,
        config,
        config_descriptor,
        bos_descriptor,
        msos_descriptor,
        control_buf,
    );

    builder.handler(device_handler);

    // Create classes on the builder.
    let config = embassy_usb::class::hid::Config {
        report_descriptor: KeyboardReport::desc(),
        request_handler: Some(request_handler),
        poll_ms: 60,
        max_packet_size: 64,
    };
    let hid = HidReaderWriter::<_, 1, 8>::new(&mut builder, state, config);

    // Build the builder.
    let usb = builder.build();

    (usb, hid)
}
