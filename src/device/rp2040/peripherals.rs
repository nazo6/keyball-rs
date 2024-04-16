use embassy_rp::peripherals::*;

pub struct Peripherals {
    pub display: DisplayPeripherals,
    pub keyboard: KeyboardPeripherals,
    pub ball: BallPeripherals,
    pub split: SplitPeripherals,
    pub led: LedPeripherals,
    pub usb: UsbPeripherals,
}

pub struct DisplayPeripherals {
    pub i2c: I2C1,
    pub scl: PIN_3,
    pub sda: PIN_2,
}

pub struct KeyboardPeripherals {
    pub row_0: PIN_4,
    pub row_1: PIN_5,
    pub row_2: PIN_6,
    pub row_3: PIN_7,
    pub row_4: PIN_8,
    pub col_0: PIN_26,
    pub col_1: PIN_27,
    pub col_2: PIN_28,
    pub col_3: PIN_29,
}

pub struct BallPeripherals {
    pub spi: BallSpiPeripherals,
    pub ncs: PIN_21,
}

pub struct BallSpiPeripherals {
    pub spi: SPI0,
    pub spi_clk: PIN_22,
    pub spi_mosi: PIN_23,
    pub spi_miso: PIN_20,
    pub spi_dma_ch0: DMA_CH0,
    pub spi_dma_ch1: DMA_CH1,
}

pub struct SplitPeripherals {
    pub pio: PIO0,
    pub data_pin: PIN_1,
    pub dma: DMA_CH3,
}

pub struct LedPeripherals {
    pub pio: PIO1,
    pub led_pin: PIN_0,
    pub dma: DMA_CH2,
}

pub struct UsbPeripherals {
    pub usb: USB,
}

pub fn init_peripherals() -> Peripherals {
    let p = embassy_rp::init(Default::default());

    Peripherals {
        display: DisplayPeripherals {
            i2c: p.I2C1,
            scl: p.PIN_3,
            sda: p.PIN_2,
        },
        keyboard: KeyboardPeripherals {
            row_0: p.PIN_4,
            row_1: p.PIN_5,
            row_2: p.PIN_6,
            row_3: p.PIN_7,
            row_4: p.PIN_8,
            col_0: p.PIN_26,
            col_1: p.PIN_27,
            col_2: p.PIN_28,
            col_3: p.PIN_29,
        },
        ball: BallPeripherals {
            spi: BallSpiPeripherals {
                spi: p.SPI0,
                spi_clk: p.PIN_22,
                spi_mosi: p.PIN_23,
                spi_miso: p.PIN_20,
                spi_dma_ch0: p.DMA_CH0,
                spi_dma_ch1: p.DMA_CH1,
            },
            ncs: p.PIN_21,
        },
        split: SplitPeripherals {
            pio: p.PIO0,
            data_pin: p.PIN_1,
            dma: p.DMA_CH3,
        },
        led: LedPeripherals {
            pio: p.PIO1,
            led_pin: p.PIN_0,
            dma: p.DMA_CH2,
        },
        usb: UsbPeripherals { usb: p.USB },
    }
}
