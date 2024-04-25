use embassy_futures::join::{join, join3};
use embassy_time::Timer;

use crate::{
    constant::MIN_SCAN_INTERVAL,
    display::DISPLAY,
    driver::{
        ball::Ball,
        keyboard::{Hand, KeyChangeEvent, KeyboardScanner},
    },
    keyboard::keymap::KEYMAP,
    state::State,
    task::{
        led_task::{LedAnimation, LedControl, LedCtrl},
        usb_task::RemoteWakeupSignal,
    },
    usb::Hid,
};

use super::super::split::*;

pub struct MasterMainLoopResource<'a, 'b> {
    pub ball: Option<Ball<'a>>,
    pub scanner: KeyboardScanner<'a>,
    pub s2m_rx: S2mRx<'b>,
    pub m2s_tx: M2sTx<'b>,
    pub led_controller: &'a LedCtrl,
    pub hid: Hid<'a>,
    pub remote_wakeup_signal: &'a RemoteWakeupSignal,
    pub hand: Hand,
}

/// Master-side main task.
pub(super) async fn start<'a, 'b>(
    MasterMainLoopResource {
        mut ball,
        mut scanner,
        s2m_rx,
        m2s_tx,
        led_controller,
        hid,
        remote_wakeup_signal,
        hand,
    }: MasterMainLoopResource<'a, 'b>,
) {
    DISPLAY.set_master(true).await;

    let (_kb_reader, mut kb_writer) = hid.keyboard.split();
    let (_mouse_reader, mut mouse_writer) = hid.mouse.split();

    let mut empty_led_sent = false;

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
                        .push(KeyChangeEvent {
                            col,
                            row,
                            pressed: true,
                        })
                        .ok();
                }
                SlaveToMaster::Released(row, col) => {
                    slave_events
                        .push(KeyChangeEvent {
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

        join3(
            async {
                state_report.report(&mut kb_writer, &mut mouse_writer).await;
            },
            async {
                if let Some(rp) = state_report.mouse_report {
                    crate::DISPLAY.set_mouse_pos(rp.x, rp.y).await;
                }
                crate::DISPLAY
                    .set_highest_layer(state_report.highest_layer)
                    .await;
            },
            async {
                // if state_report.highest_layer == 1 {
                //     let led = LedControl::Start(LedAnimation::SolidColor(50, 0, 0));
                //     led_controller.signal(led.clone());
                //     let _ = m2s_tx.try_send(MasterToSlave::Led(led));
                //     empty_led_sent = false;
                // } else if !empty_led_sent {
                //     let led = LedControl::Start(LedAnimation::SolidColor(0, 0, 0));
                //     led_controller.signal(led.clone());
                //     let _ = m2s_tx.try_send(MasterToSlave::Led(led));
                //     empty_led_sent = true;
                // }
            },
        )
        .await;

        let took = start.elapsed().as_micros();
        // crate::print!("{} {} {}\n", d1, d2, took);
        if took < MIN_SCAN_INTERVAL {
            Timer::after_micros(MIN_SCAN_INTERVAL * 1000 - took).await;
        }
    }
}
