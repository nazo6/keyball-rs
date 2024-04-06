use embassy_rp::{
    bind_interrupts,
    peripherals::PIO1,
    pio::{InterruptHandler, Pio},
};
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    channel::{Channel, Receiver, Sender},
};

use crate::input::led::ws2812::Ws2812;

use super::LedPeripherals;

use smart_leds::RGB8;

mod ws2812;

bind_interrupts!(struct Irqs {
    PIO1_IRQ_0 => InterruptHandler<PIO1>;
});

pub struct LedControl;

pub type LedCtrlChannel = Channel<ThreadModeRawMutex, LedControl, 1>;
pub type LedCtrlRx<'a> = Receiver<'a, ThreadModeRawMutex, LedControl, 1>;
pub type LedCtrlTx<'a> = Sender<'a, ThreadModeRawMutex, LedControl, 1>;

pub async fn start(p: LedPeripherals, led_ctrl_rx: LedCtrlRx<'_>) {
    let Pio {
        mut common, sm0, ..
    } = Pio::new(p.pio, Irqs);

    // This is the number of leds in the string. Helpfully, the sparkfun thing plus and adafruit
    // feather boards for the 2040 both have one built in.
    const NUM_LEDS: usize = 1;
    let mut data = [RGB8::default(); NUM_LEDS];
    data[0] = (50, 100, 100).into();

    // Common neopixel pins:
    // Thing plus: 8
    // Adafruit Feather: 16;  Adafruit Feather+RFM95: 4
    let mut ws2812: Ws2812<'_, PIO1, 0, 1> = Ws2812::new(&mut common, sm0, p.dma, p.led_pin);

    ws2812.write(&data).await;
}
