use embassy_futures::join::{join, join3};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, channel::Sender};
use embassy_time::Timer;
use usbd_hid::descriptor::{KeyboardReport, MediaKeyboardReport, MouseReport};

use crate::{
    constant::MIN_KB_SCAN_INTERVAL,
    display::DISPLAY,
    driver::{
        ball::Ball,
        keyboard::{Hand, KeyChangeEventOneHand, KeyboardScanner},
    },
    keyboard::keymap::KEYMAP,
    state::State,
    task::led_task::{LedAnimation, LedControl, LedCtrl},
};

use super::super::split::*;

pub struct MasterMainLoopResource<'a, 'b> {
    pub ball: Option<Ball<'a>>,
    pub scanner: KeyboardScanner<'a>,
    pub s2m_rx: S2mRx<'b>,
    pub m2s_tx: M2sTx<'b>,
    pub led_controller: &'a LedCtrl,
    pub hand: Hand,
    pub kb_report_tx: Sender<'a, ThreadModeRawMutex, KeyboardReport, 10>,
    pub mouse_report_tx: Sender<'a, ThreadModeRawMutex, MouseReport, 10>,
    pub mkb_report_tx: Sender<'a, ThreadModeRawMutex, MediaKeyboardReport, 10>,
}

/// Master-side main task.
pub(super) async fn start<'a, 'b>(
    MasterMainLoopResource {
        mut ball,
        mut scanner,
        s2m_rx,
        m2s_tx,
        led_controller,
        hand,
        kb_report_tx,
        mouse_report_tx,
        mkb_report_tx,
    }: MasterMainLoopResource<'a, 'b>,
) {
    DISPLAY.set_master(true).await;

    let mut latest_led: Option<LedControl> = None;

    let mut slave_events = heapless::Vec::<_, 16>::new();

    let mut state = State::new(KEYMAP, hand);

    loop {
        slave_events.clear();

        let start = embassy_time::Instant::now();

        let mut mouse: (i8, i8) = (0, 0);

        while let Ok(cmd_from_slave) = s2m_rx.try_receive() {
            match cmd_from_slave {
                SlaveToMaster::Pressed(row, col) => {
                    slave_events
                        .push(KeyChangeEventOneHand {
                            col,
                            row,
                            pressed: true,
                        })
                        .ok();
                }
                SlaveToMaster::Released(row, col) => {
                    slave_events
                        .push(KeyChangeEventOneHand {
                            col,
                            row,
                            pressed: false,
                        })
                        .ok();
                }
                SlaveToMaster::Mouse { x, y } => {
                    mouse.0 += x;
                    mouse.1 += y;
                }
                SlaveToMaster::Message(_) => {}
            }
        }

        let (mut master_events, _) = join(async { scanner.scan().await }, async {
            if let Some(ball) = &mut ball {
                if let Ok(Some((x, y))) = ball.read().await {
                    mouse.0 += x;
                    mouse.1 += y;
                }
            }
        })
        .await;

        let state_report = state.update(&mut master_events, &mut slave_events, &mouse);

        join(
            async {
                if let Some(rp) = state_report.mouse_report {
                    crate::DISPLAY.set_mouse_pos(rp.x, rp.y).await;
                }
                crate::DISPLAY
                    .set_highest_layer(state_report.highest_layer)
                    .await;
            },
            async {
                let led = match state_report.highest_layer {
                    1 => LedControl::Start(LedAnimation::SolidColor(0, 0, 1)),
                    2 => LedControl::Start(LedAnimation::SolidColor(1, 0, 0)),
                    3 => LedControl::Start(LedAnimation::SolidColor(0, 25, 2)),
                    _ => LedControl::Start(LedAnimation::SolidColor(0, 0, 0)),
                };

                if let Some(latest_led) = &latest_led {
                    if led != *latest_led {
                        led_controller.signal(led.clone());
                        let _ = m2s_tx.try_send(MasterToSlave::Led(led.clone()));
                    }
                }

                latest_led = Some(led);
            },
        )
        .await;

        join3(
            async {
                if let Some(report) = state_report.keyboard_report {
                    let _ = kb_report_tx.try_send(report);
                }
            },
            async {
                if let Some(report) = state_report.mouse_report {
                    let _ = mouse_report_tx.try_send(report);
                }
            },
            async {
                if let Some(report) = state_report.media_keyboard_report {
                    let _ = mkb_report_tx.try_send(report);
                }
            },
        )
        .await;

        let took = start.elapsed();
        if took < MIN_KB_SCAN_INTERVAL {
            Timer::after(MIN_KB_SCAN_INTERVAL - took).await;
        }
    }
}
