use core::fmt::Write;

use embassy_futures::join::join;
use embassy_time::Timer;
use usbd_hid::descriptor::{KeyboardReport, MouseReport};

use crate::DISPLAY;

use super::{
    ball::{self, Ball},
    keyboard::{self, Keyboard},
    split::{self, M2sRx, M2sTx, S2mRx, S2mTx},
    Hid,
};

async fn read_report(
    keyboard: &mut keyboard::Keyboard<'_>,
    ball: Option<&mut ball::Ball<'_>>,
    other_side_keys: &[Option<(u8, u8)>; 6],
) -> (Option<KeyboardReport>, Option<MouseReport>) {
    if let Some(ball) = ball {
        let (ball, keyboard) = join(ball.read(), keyboard.read(other_side_keys)).await;
        (keyboard, ball)
    } else {
        let keyboard = keyboard.read(other_side_keys).await;
        (keyboard, None)
    }
}

pub async fn main_master_task(
    hid: Hid<'_>,
    mut ball: Option<Ball<'_>>,
    mut keyboard: Keyboard<'_>,
    s2m_rx: S2mRx<'_>,
    m2s_tx: M2sTx<'_>,
) {
    let (kb_reader, mut kb_writer) = hid.keyboard.split();
    let (_mouse_reader, mut mouse_writer) = hid.mouse.split();

    DISPLAY.lock().await.as_mut().unwrap().draw_text("master");

    let mut empty_kb_sent = false;
    let mut other_side_keys = [None; 6];
    loop {
        while let Ok(cmd_from_slave) = s2m_rx.try_receive() {
            match cmd_from_slave {
                split::SlaveToMaster::Pressed { keys } => {
                    other_side_keys = keys;
                }
                split::SlaveToMaster::Message(_) => {}
                _ => {}
            }
        }

        // master
        let (kb_report, ball) = read_report(&mut keyboard, ball.as_mut(), &other_side_keys).await;

        join(
            async {
                if let Some(kb_report) = kb_report {
                    let _ = kb_writer.write_serialize(&kb_report).await;
                    empty_kb_sent = false;
                } else if !empty_kb_sent {
                    let _ = kb_writer
                        .write_serialize(&usbd_hid::descriptor::KeyboardReport {
                            keycodes: [0; 6],
                            leds: 0,
                            modifier: 0,
                            reserved: 0,
                        })
                        .await;
                    empty_kb_sent = true;
                }
            },
            async {
                if let Some(mouse_report) = ball {
                    let _ = mouse_writer.write_serialize(&mouse_report).await;
                }
            },
        )
        .await;

        Timer::after_millis(10).await;
    }
}

pub async fn main_slave_task(
    mut ball: Option<Ball<'_>>,
    mut keyboard: Keyboard<'_>,
    m2s_rx: M2sRx<'_>,
    s2m_tx: S2mTx<'_>,
) {
    DISPLAY.lock().await.as_mut().unwrap().draw_text("slave");

    let mut pressed_keys_prev = [None; 6];
    loop {
        let pressed = keyboard.read_matrix().await;
        let mut pressed_keys = [None; 6];
        for (i, (row, col)) in pressed.iter().enumerate() {
            if i >= pressed_keys.len() {
                break;
            }
            pressed_keys[i] = Some((*row, *col));
        }
        if pressed_keys != pressed_keys_prev {
            s2m_tx
                .send(split::SlaveToMaster::Pressed { keys: pressed_keys })
                .await;
            pressed_keys_prev = pressed_keys;
        }

        Timer::after_millis(10).await;
    }
}
