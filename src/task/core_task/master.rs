use embassy_futures::join::{join, join4};
use embassy_time::Timer;
use usbd_hid::descriptor::MouseReport;

use crate::{
    constant::MIN_SCAN_INTERVAL,
    display::DISPLAY,
    driver::{ball::Ball, keyboard::KeyboardScanner},
    keyboard::{keymap::KEYMAP, pressed::Pressed, state::KeyboardState},
    task::{
        led_task::{LedAnimation, LedControl, LedCtrl},
        usb_task::RemoteWakeupSignal,
    },
    usb::{Hid, SUSPENDED},
};

use super::split::{M2sTx, MasterToSlave, S2mRx, SlaveToMaster};

/// Master-side main task.
pub async fn start(
    mut ball: Option<Ball<'_>>,
    mut scanner: KeyboardScanner<'_>,
    s2m_rx: S2mRx<'_>,
    m2s_tx: M2sTx<'_>,
    led_controller: &LedCtrl,
    hid: Hid<'_>,
    remote_wakeup_signal: &RemoteWakeupSignal,
) {
    DISPLAY.set_master(true).await;

    let (_kb_reader, mut kb_writer) = hid.keyboard.split();
    let (_mouse_reader, mut mouse_writer) = hid.mouse.split();

    let mut empty_kb_sent = false;
    let mut empty_mouse_sent = false;
    let mut empty_led_sent = false;

    let mut master_pressed = Pressed::new();
    let mut slave_pressed = Pressed::new();

    let mut kb_state = KeyboardState::new(KEYMAP, scanner.hand);

    loop {
        let start = embassy_time::Instant::now();

        let mut mouse: Option<(i8, i8)> = None;

        while let Ok(cmd_from_slave) = s2m_rx.try_receive() {
            match cmd_from_slave {
                SlaveToMaster::Pressed(row, col) => {
                    slave_pressed.set_pressed(true, row, col);
                }
                SlaveToMaster::Released(row, col) => {
                    slave_pressed.set_pressed(false, row, col);
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
                scanner.scan_and_update(&mut master_pressed).await;
                kb_state.update_and_report(&master_pressed, &slave_pressed)
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

        join4(
            async {
                if !key_status.empty_keyboard_report {
                    if SUSPENDED.load(core::sync::atomic::Ordering::Relaxed) {
                        remote_wakeup_signal.signal(());
                    }

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
                    // なんか知らんけど逆
                    mouse_report.x = y;
                    mouse_report.y = x;
                } else if key_status.mouse_button == 0 {
                    return;
                    // if !empty_mouse_sent {
                    //     let _ = mouse_writer.write_serialize(&mouse_report).await;
                    //     empty_mouse_sent = true;
                    // } else {
                    //     return;
                    // }
                }

                mouse_report.buttons = key_status.mouse_button;

                let _ = mouse_writer.write_serialize(&mouse_report).await;
                empty_mouse_sent = false;
            },
            async {
                if let Some((x, y)) = mouse_status {
                    crate::DISPLAY.set_mouse_pos(x, y).await;
                }
                crate::DISPLAY
                    .set_highest_layer(key_status.highest_layer as u8)
                    .await;
            },
            async {
                if key_status.highest_layer == 1 {
                    let led = LedControl::Start(LedAnimation::SolidColor(50, 0, 0));
                    led_controller.signal(led.clone());
                    let _ = m2s_tx.try_send(MasterToSlave::Led(led));
                    empty_led_sent = false;
                } else if !empty_led_sent {
                    let led = LedControl::Start(LedAnimation::SolidColor(0, 0, 0));
                    led_controller.signal(led.clone());
                    let _ = m2s_tx.try_send(MasterToSlave::Led(led));
                    empty_led_sent = true;
                }
            },
        )
        .await;

        let took = start.elapsed().as_millis();
        crate::utils::print!("Took: {}    ", start.elapsed().as_micros());
        if took < MIN_SCAN_INTERVAL {
            Timer::after_millis(MIN_SCAN_INTERVAL - took).await;
        }
    }
}
