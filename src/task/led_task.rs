use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    channel::{Channel, Receiver, Sender},
};

use rkyv::{Archive, Deserialize, Serialize};
use smart_leds::RGB8;

use crate::driver::led::Ws2812;

use super::LedPeripherals;

#[derive(Archive, Deserialize, Serialize, Debug)]
pub enum LedControl {
    Animation(LedAnimation),
    Reset,
}

#[derive(Archive, Deserialize, Serialize, Debug)]
pub enum LedAnimation {
    Rainbow,
    Blink,
    SolidColor(u8, u8, u8),
}

pub type LedCtrlChannel = Channel<ThreadModeRawMutex, LedControl, 1>;
pub type LedCtrlRx<'a> = Receiver<'a, ThreadModeRawMutex, LedControl, 1>;
pub type LedCtrlTx<'a> = Sender<'a, ThreadModeRawMutex, LedControl, 1>;

pub async fn start(p: LedPeripherals, led_ctrl_rx: LedCtrlRx<'_>) {
    // TODO: led_ctrl_rxから受け取ったメッセージによってLEDの色を変える

    const NUM_LEDS: usize = 1;

    let mut ws2812: Ws2812 = Ws2812::new(p);

    loop {
        let ctrl = led_ctrl_rx.receive().await;
        match ctrl {
            LedControl::Animation(led_animation) => {
                match led_animation {
                    LedAnimation::Rainbow => {
                        //
                    }
                    LedAnimation::Blink => {
                        //
                    }
                    LedAnimation::SolidColor(r, g, b) => {
                        let data = [(r, g, b).into(); NUM_LEDS];
                        ws2812.write(&data).await;
                    }
                }
            }
            LedControl::Reset => {
                let data = [RGB8::default(); NUM_LEDS];
                ws2812.write(&data).await;
            }
        }
    }
}
