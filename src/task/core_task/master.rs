use embassy_futures::join::join;
use embassy_time::Timer;
use usbd_hid::descriptor::MouseReport;

use crate::{
    constant::MIN_SCAN_INTERVAL,
    display::DISPLAY,
    driver::{ball::Ball, keyboard::Keyboard},
    keyboard::{keymap::KEYMAP, pressed::Pressed, state::KeyboardState},
    usb::Hid,
    utils::print,
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

    let hand = keyboard.get_hand().await;
    DISPLAY.set_hand(hand).await;

    let (kb_reader, mut kb_writer) = hid.keyboard.split();
    let (_mouse_reader, mut mouse_writer) = hid.mouse.split();

    let mut empty_kb_sent = false;
    let mut slave_keys = [None; 6];

    let mut pressed = Pressed::new(hand);
    let mut kb_state = KeyboardState::new(KEYMAP);

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

        let (key_status, mouse_status) = join(
            async {
                keyboard.scan_and_update(&mut pressed).await;
                kb_state.update_and_report(&pressed, &slave_keys)
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
                mouse
            },
        )
        .await;

        join(
            async {
                if !key_status.empty_keyboard_report {
                    let _ = kb_writer.write_serialize(&key_status.keyboard_report).await;
                    empty_kb_sent = false;
                } else if !empty_kb_sent {
                    let _ = kb_writer.write_serialize(&key_status.keyboard_report).await;
                    empty_kb_sent = true;
                }
            },
            async {
                let mut mouse_report = MouseReport {
                    x: 0,
                    y: 0,
                    buttons: 0,
                    pan: 0,
                    wheel: 0,
                };

                if let Some((x, y)) = mouse_status {
                    mouse_report.x = y;
                    mouse_report.y = x;
                } else if key_status.mouse_button == 0 {
                    return;
                }

                mouse_report.buttons = key_status.mouse_button;

                let _ = mouse_writer.write_serialize(&mouse_report).await;
            },
        )
        .await;

        let took = start.elapsed().as_millis();
        if took < MIN_SCAN_INTERVAL {
            Timer::after_millis(MIN_SCAN_INTERVAL - took).await;
        }
    }
}
