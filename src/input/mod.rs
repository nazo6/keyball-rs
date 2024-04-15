use embassy_futures::join::join;
use embassy_rp::{peripherals::*, usb::Driver};
use embassy_sync::channel::Channel;
use embassy_usb::class::hid::HidReaderWriter;

mod ball;
mod keyboard;
mod led;
mod main_task;
mod split;

pub struct InputPeripherals {
    pub keyboard: KeyboardPeripherals,
    pub ball: BallPeripherals,
    pub split: SplitInputPeripherals,
    pub led: LedPeripherals,
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
    pub spi: SPI0,
    pub spi_clk: PIN_22,
    pub spi_mosi: PIN_23,
    pub spi_miso: PIN_20,
    pub spi_dma_ch0: DMA_CH0,
    pub spi_dma_ch1: DMA_CH1,
    pub ncs: PIN_21,
}

pub struct SplitInputPeripherals {
    pub pio: PIO0,
    pub data_pin: PIN_1,
    pub dma: DMA_CH3,
}

pub struct LedPeripherals {
    pub pio: PIO1,
    pub led_pin: PIN_0,
    pub dma: DMA_CH2,
}

pub struct Hid<'a> {
    pub keyboard: HidReaderWriter<'a, Driver<'a, USB>, 1, 8>,
    pub mouse: HidReaderWriter<'a, Driver<'a, USB>, 1, 8>,
}

/// Starts the input task.
/// If hid is Some, this is master side, and report will be sent to the USB device.
/// If hid is None, this is slave side, and report will be sent to the master.
pub async fn start(peripherals: InputPeripherals, hid: Option<Hid<'_>>) {
    // let hid = if let Some(hid) = hid {
    //     // TODO: handle keyboard reader
    //     let (kb_reader, kb_writer) = hid.keyboard.split();
    //     let (_mouse_reader, mouse_writer) = hid.mouse.split();
    //     Some((kb_writer, mouse_writer))
    // } else {
    //     None
    // };

    let hid = hid.unwrap();

    let s2m_chan: split::S2mChannel = Channel::new();
    let s2m_tx = s2m_chan.sender();
    let s2m_rx = s2m_chan.receiver();

    let m2s_chan: split::M2sChannel = Channel::new();
    let m2s_tx = m2s_chan.sender();
    let m2s_rx = m2s_chan.receiver();

    let led_ctrl_chan: led::LedCtrlChannel = Channel::new();
    let led_ctrl_rx = led_ctrl_chan.receiver();
    let led_ctrl_tx = led_ctrl_chan.sender();

    let ball = ball::Ball::init(peripherals.ball).await.ok();
    let keyboard = keyboard::Keyboard::new(peripherals.keyboard);

    join(
        async {
            // TODO: masterとslaveをちゃんと判定する
            match &ball {
                Some(_) => {
                    join(
                        split::master_split_handle(peripherals.split, m2s_rx, s2m_tx),
                        main_task::main_master_task(hid, ball, keyboard, s2m_rx, m2s_tx),
                    )
                    .await
                }
                None => {
                    join(
                        split::slave_split_handle(peripherals.split, m2s_tx, s2m_rx),
                        main_task::main_slave_task(ball, keyboard, m2s_rx, s2m_tx),
                    )
                    .await
                }
            }
        },
        led::start(peripherals.led, led_ctrl_rx),
    )
    .await;
}
