use embassy_rp::{peripherals::PIO1, pio::Pio};
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    channel::{Channel, Receiver, Sender},
};

use smart_leds::RGB8;

use crate::{device::interrupts::Irqs, driver::led::Ws2812};

use super::LedPeripherals;

pub struct LedControl;

pub type LedCtrlChannel = Channel<ThreadModeRawMutex, LedControl, 1>;
pub type LedCtrlRx<'a> = Receiver<'a, ThreadModeRawMutex, LedControl, 1>;
pub type LedCtrlTx<'a> = Sender<'a, ThreadModeRawMutex, LedControl, 1>;

pub async fn start(p: LedPeripherals, led_ctrl_rx: LedCtrlRx<'_>) {
    // TODO: led_ctrl_rxから受け取ったメッセージによってLEDの色を変える

    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.pio, Irqs);

    const NUM_LEDS: usize = 1;
    let mut data = [RGB8::default(); NUM_LEDS];
    data[0] = (50, 100, 100).into();

    let mut ws2812: Ws2812<'_, PIO1, 0, 1> = Ws2812::new(&mut common, sm0, p.dma, p.led_pin);

    // ws2812.write(&data).await;
}
