pub mod peripherals;

pub mod interrupts {
    use embassy_rp::{
        bind_interrupts,
        peripherals::{I2C1, PIO0, PIO1, USB},
    };

    bind_interrupts!(pub struct Irqs {
        PIO0_IRQ_0 => embassy_rp::pio::InterruptHandler<PIO0>;
        PIO1_IRQ_0 => embassy_rp::pio::InterruptHandler<PIO1>;
        USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<USB>;
        I2C1_IRQ => embassy_rp::i2c::InterruptHandler<I2C1>;
        ADC_IRQ_FIFO => embassy_rp::adc::InterruptHandler;
    });
}

pub mod i2c_display {
    use embassy_rp::{i2c::Async, peripherals::I2C1};

    use super::interrupts::Irqs;

    pub type I2C<'a> = embassy_rp::i2c::I2c<'a, I2C1, Async>;

    pub fn create_i2c<'a>(p: super::peripherals::DisplayPeripherals, frequency: u32) -> I2C<'a> {
        let mut i2c_config = embassy_rp::i2c::Config::default();
        i2c_config.frequency = frequency;

        embassy_rp::i2c::I2c::new_async(p.i2c, p.scl, p.sda, Irqs, i2c_config)
    }
}

pub mod spi_ball {
    use embassy_rp::{
        peripherals::SPI0,
        spi::{Async, Spi as RpSpi},
    };

    use super::peripherals::BallSpiPeripherals;

    pub type SpiPeripheral = SPI0;
    pub type Spi<'a> = RpSpi<'a, SpiPeripheral, Async>;
    pub type SpiError = embassy_rp::spi::Error;

    pub fn create_ball_spi(p: BallSpiPeripherals) -> Spi<'static> {
        let mut spi_config = embassy_rp::spi::Config::default();
        spi_config.frequency = 2_000_000;
        spi_config.polarity = embassy_rp::spi::Polarity::IdleHigh;
        spi_config.phase = embassy_rp::spi::Phase::CaptureOnSecondTransition;

        Spi::new(
            p.spi,
            p.spi_clk,
            p.spi_mosi,
            p.spi_miso,
            p.spi_dma_ch0,
            p.spi_dma_ch1,
            spi_config,
        )
    }
}

pub mod usb {
    use embassy_rp::{peripherals::USB, usb::Driver};
    use embassy_usb::UsbDevice;

    use super::{interrupts::Irqs, peripherals::UsbPeripherals};

    pub type DeviceDriver<'a> = Driver<'a, USB>;

    pub fn create_usb_driver<'a>(p: UsbPeripherals) -> Driver<'a, USB> {
        Driver::new(p.usb, Irqs)
    }

    // ref: https://github.com/rp-rs/rp-hal/blob/a1b20f3a2cc0702986c478b0e1ee666f44d66853/rp2040-hal/src/usb.rs#L494
    // https://docs.rs/rp-pac/6.0.0/rp_pac/usb/regs/struct.SieCtrl.html
    // TODO: クリティカルセッションのことを考慮しないとだめかも？
    // まあそんなに同時に書き込まれることはないだろうしとりあえず…
    pub async fn remote_wakeup<'a>(_device: &mut UsbDevice<'a, DeviceDriver<'a>>) {
        embassy_rp::pac::USBCTRL_REGS
            .sie_ctrl()
            .modify(|w| w.set_resume(true));
    }
}

pub mod gpio {
    pub use embassy_rp::gpio::*;
}

pub mod adc {
    use embassy_rp::{
        adc::{Adc, Async, Channel, Config},
        peripherals::{ADC, ADC_TEMP_SENSOR},
    };

    use super::interrupts::Irqs;

    pub fn create_adc<'a>(adc_peripheral: ADC) -> Adc<'a, Async> {
        Adc::new(adc_peripheral, Irqs, Config::default())
    }

    pub fn temp_sensor_channel<'a>(adc_temp_sensor_peripheral: ADC_TEMP_SENSOR) -> Channel<'a> {
        Channel::new_temp_sensor(adc_temp_sensor_peripheral)
    }
}
