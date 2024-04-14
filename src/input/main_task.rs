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
) -> (Option<KeyboardReport>, Option<MouseReport>) {
    if let Some(ball) = ball {
        let (ball, keyboard) = join(ball.read(), keyboard.read()).await;
        (keyboard, ball)
    } else {
        let keyboard = keyboard.read().await;
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
    loop {
        // master
        let (kb_report, ball) = read_report(&mut keyboard, ball.as_mut()).await;

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

        while let Ok(cmd_from_slave) = s2m_rx.try_receive() {
            //
        }

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

    loop {
        // slave
        let (kb_report, ball) = read_report(&mut keyboard, ball.as_mut()).await;
        if let Some(kb_report) = kb_report {
            if let Some(kc) = kb_report.keycodes.first() {
                s2m_tx.send(split::SlaveToMaster::Message(*kc)).await;
            }
        }

        Timer::after_millis(10).await;
    }
}
