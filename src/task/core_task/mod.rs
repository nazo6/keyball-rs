use embassy_futures::{
    join::join,
    select::{select, Either},
};
use embassy_sync::channel::Channel;
use embassy_time::Timer;

use crate::{
    driver::{ball, keyboard},
    usb::Hid,
};

use super::{led_task::LedCtrlTx, BallPeripherals, KeyboardPeripherals, SplitPeripherals};

mod master;
mod slave;
mod utils;

mod split;

pub async fn start(
    ball_peripherals: BallPeripherals,
    keyboard_peripherals: KeyboardPeripherals,
    split_peripherals: SplitPeripherals,
    led_controller: LedCtrlTx<'_>,
    mut hid: Hid<'_>,
) {
    // If usb connection is ready, this is master side.
    let is_master = match select(hid.keyboard.ready(), Timer::after_secs(2)).await {
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
    let keyboard = keyboard::Keyboard::new(keyboard_peripherals);

    if is_master {
        join(
            split::master_split_handle(split_peripherals, m2s_rx, s2m_tx),
            master::main_master_task(hid, ball, keyboard, s2m_rx, m2s_tx),
        )
        .await;
    } else {
        join(
            split::slave_split_handle(split_peripherals, m2s_tx, s2m_rx),
            slave::main_slave_task(ball, keyboard, m2s_rx, s2m_tx),
        )
        .await;
    }
}
