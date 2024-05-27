use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};

use serde::{Deserialize, Serialize};
use smart_leds::RGB8;

use crate::{
    config::{LEFT_LED_NUM, RIGHT_LED_NUM},
    driver::{keyboard::Hand, led::Ws2812},
};

use super::LedPeripherals;

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub enum LedControl {
    Start(LedAnimation),
    Reset,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub enum LedAnimation {
    Rainbow,
    Blink,
    // Color (rgb)
    SolidColor(u8, u8, u8),
}

pub type LedCtrl = Signal<ThreadModeRawMutex, LedControl>;

pub struct LedTaskResource<'a> {
    pub peripherals: LedPeripherals,
    pub led_ctrl: &'a LedCtrl,
    pub hand: Hand,
}
pub async fn start(
    LedTaskResource {
        peripherals: p,
        led_ctrl,
        hand,
    }: LedTaskResource<'_>,
) {
    let mut ws2812: Ws2812 = Ws2812::new(p);

    loop {
        let ctrl = led_ctrl.wait().await;
        match ctrl {
            LedControl::Start(led_animation) => {
                match led_animation {
                    LedAnimation::Rainbow => {
                        //
                    }
                    LedAnimation::Blink => {
                        //
                    }
                    LedAnimation::SolidColor(r, g, b) => {
                        let color = (r, g, b).into();
                        match hand {
                            Hand::Left => {
                                let data = [color; LEFT_LED_NUM];
                                ws2812.write(&data).await;
                            }
                            Hand::Right => {
                                let data = [color; RIGHT_LED_NUM];
                                ws2812.write(&data).await;
                            }
                        }
                    }
                }
            }
            LedControl::Reset => match hand {
                Hand::Left => {
                    let data = [RGB8::default(); LEFT_LED_NUM];
                    ws2812.write(&data).await;
                }
                Hand::Right => {
                    let data = [RGB8::default(); RIGHT_LED_NUM];
                    ws2812.write(&data).await;
                }
            },
        }
    }
}
