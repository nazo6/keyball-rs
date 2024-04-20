use embassy_futures::{
    join::join,
    select::{select, Either},
};
use embassy_sync::channel::Channel;
use embassy_time::Timer;

use crate::{
    constant::SPLIT_USB_TIMEOUT,
    driver::{ball, keyboard::KeyboardScanner},
    usb::Hid,
};

use super::{led_task::LedCtrl, usb_task::RemoteWakeupSignal, BallPeripherals, SplitPeripherals};

mod master;
mod slave;

mod split;

pub struct CoreTaskResource<'a> {
    pub ball_peripherals: BallPeripherals,
    pub split_peripherals: SplitPeripherals,
    pub scanner: KeyboardScanner<'a>,
    pub led_controller: &'a LedCtrl,
    pub hid: Hid<'a>,
    pub remote_wakeup_signal: &'a RemoteWakeupSignal,
}

pub async fn start(
    CoreTaskResource {
        ball_peripherals,
        split_peripherals,
        scanner,
        led_controller,
        mut hid,
        remote_wakeup_signal,
    }: CoreTaskResource<'_>,
) {
    // VBUS detection is not available for ProMicro RP2040, so USB communication is used to determine master/slave.
    // This is same as SPLIT_USB_DETECT in QMK.
    let is_master = match select(hid.keyboard.ready(), Timer::after_millis(SPLIT_USB_TIMEOUT)).await
    {
        Either::First(_) => true,
        Either::Second(_) => false,
    };

    let s2m_chan: split::S2mChannel = Channel::new();
    let s2m_tx = s2m_chan.sender();
    let s2m_rx = s2m_chan.receiver();

    let m2s_chan: split::M2sChannel = Channel::new();
    let m2s_tx = m2s_chan.sender();
    let m2s_rx = m2s_chan.receiver();

    let ball = ball::Ball::init(ball_peripherals).await.ok();

    crate::DISPLAY.set_mouse(ball.is_some()).await;

    #[cfg(feature = "force-master")]
    {
        join(
            master::start(hid, ball, keyboard, s2m_rx, m2s_tx),
            split::master_split_handle(split_peripherals, m2s_rx, s2m_tx),
        )
        .await;
        return;
    }

    #[cfg(feature = "force-slave")]
    {
        join(
            slave::start(ball, keyboard, m2s_rx, s2m_tx),
            split::slave_split_handle(split_peripherals, m2s_tx, s2m_rx),
        )
        .await;
        return;
    }

    if is_master {
        join(
            master::start(
                ball,
                scanner,
                s2m_rx,
                m2s_tx,
                led_controller,
                hid,
                remote_wakeup_signal,
            ),
            split::master_split_handle(split_peripherals, m2s_rx, s2m_tx),
        )
        .await;
    } else {
        join(
            slave::start(ball, scanner, m2s_rx, s2m_tx, led_controller),
            split::slave_split_handle(split_peripherals, m2s_tx, s2m_rx),
        )
        .await;
    }
}
