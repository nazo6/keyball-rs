use embassy_futures::join::join;
use embassy_time::Timer;
use usbd_hid::descriptor::{KeyboardReport, MouseReport};

use crate::{
    constant::MIN_SCAN_INTERVAL,
    display::DISPLAY,
    driver::{
        ball::Ball,
        keyboard::{pressed::Pressed, Keyboard},
    },
    keyconfig::keycodes::KC_NO,
    usb::Hid,
};

use super::split::{M2sTx, S2mRx, SlaveToMaster};

/// Master-side main task.
pub async fn start(
    hid: Hid<'_>,
    mut ball: Option<Ball<'_>>,
    mut keyboard: Keyboard<'_>,
    s2m_rx: S2mRx<'_>,
    m2s_tx: M2sTx<'_>,
) {
    DISPLAY.set_master(true).await;

    let (kb_reader, mut kb_writer) = hid.keyboard.split();
    let (_mouse_reader, mut mouse_writer) = hid.mouse.split();

    let mut empty_kb_sent = false;
    let mut keyboard_state = Pressed::new();
    let mut slave_keys = [None; 6];

    let mut kb_report = KeyboardReport {
        keycodes: [0; 6],
        leds: 0,
        modifier: 0,
        reserved: 0,
    };

    loop {
        let start = embassy_time::Instant::now();

        let mut mouse: Option<(i8, i8)> = None;

        if let Ok(cmd_from_slave) = s2m_rx.try_receive() {
            match cmd_from_slave {
                SlaveToMaster::Pressed { keys } => {
                    slave_keys = keys;
                }
                SlaveToMaster::Mouse { x, y } => {
                    if let Some(mouse) = &mut mouse {
                        mouse.0 += x;
                        mouse.1 += y;
                    } else {
                        mouse = Some((x, y));
                    }
                }
                SlaveToMaster::Message(_) => {}
            }
        }

        join(
            async {
                let mut keycodes = [0; 6];
                let mut idx = 0;

                for (row, col) in slave_keys.iter().flatten() {
                    if let Some(kc) = keyboard_state.get_keycode(*row, *col) {
                        if kc == KC_NO {
                            continue;
                        }
                        if idx >= keycodes.len() {
                            break;
                        }
                        keycodes[idx] = kc;
                        idx += 1;
                    }
                }

                keyboard.scan_and_update(&mut keyboard_state).await;

                for (row, col) in keyboard_state.iter() {
                    if let Some(kc) = keyboard_state.get_keycode(row, col) {
                        if idx >= keycodes.len() {
                            break;
                        }
                        keycodes[idx] = kc;
                        idx += 1;
                    }
                }

                if idx > 0 {
                    kb_report.keycodes = keycodes;
                    let _ = kb_writer.write_serialize(&kb_report).await;
                    empty_kb_sent = false;
                } else if !empty_kb_sent {
                    kb_report.keycodes = [0; 6];
                    let _ = kb_writer.write_serialize(&kb_report).await;
                    empty_kb_sent = true;
                }
            },
            async {
                if let Some(ball) = &mut ball {
                    if let Ok(Some((x, y))) = ball.read().await {
                        if let Some(mouse) = &mut mouse {
                            mouse.0 += x;
                            mouse.1 += y;
                        } else {
                            mouse = Some((x, y));
                        }
                    }
                }

                if let Some(mouse) = mouse {
                    let _ = mouse_writer
                        .write_serialize(&MouseReport {
                            buttons: 0,
                            // Mouse x and y are swapped
                            x: mouse.1,
                            y: mouse.0,
                            wheel: 0,
                            pan: 0,
                        })
                        .await;
                }
            },
        )
        .await;

        let took = start.elapsed().as_millis();
        if took < MIN_SCAN_INTERVAL {
            Timer::after_millis(MIN_SCAN_INTERVAL - took).await;
        }
    }
}
