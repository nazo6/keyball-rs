use embassy_futures::join::join;
use embassy_time::Timer;

use crate::{
    display::DISPLAY,
    driver::{ball::Ball, keyboard::Keyboard},
    usb::Hid,
};

use super::{
    split::{M2sTx, S2mRx, SlaveToMaster},
    utils::read_report,
};

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
                SlaveToMaster::Pressed { keys } => {
                    other_side_keys = keys;
                }
                SlaveToMaster::Message(_) => {}
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
