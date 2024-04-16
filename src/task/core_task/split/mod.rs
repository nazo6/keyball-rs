use core::fmt::Write;

use crate::constant::SPLIT_CHANNEL_SIZE;
use crate::device::interrupts::Irqs;
use crate::display::DISPLAY;
use embassy_futures::select::{select, Either};
use embassy_rp::pio::Pio;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};

use self::communicate::Communicate;

use super::SplitPeripherals;

mod communicate;
mod data;
pub use data::*;

pub type S2mChannel = Channel<ThreadModeRawMutex, SlaveToMaster, SPLIT_CHANNEL_SIZE>;
pub type S2mRx<'a> = Receiver<'a, ThreadModeRawMutex, SlaveToMaster, SPLIT_CHANNEL_SIZE>;
pub type S2mTx<'a> = Sender<'a, ThreadModeRawMutex, SlaveToMaster, SPLIT_CHANNEL_SIZE>;

pub type M2sChannel = Channel<ThreadModeRawMutex, MasterToSlave, SPLIT_CHANNEL_SIZE>;
pub type M2sRx<'a> = Receiver<'a, ThreadModeRawMutex, MasterToSlave, SPLIT_CHANNEL_SIZE>;
pub type M2sTx<'a> = Sender<'a, ThreadModeRawMutex, MasterToSlave, SPLIT_CHANNEL_SIZE>;

//
// Send data to slave
//
//  ┌─main loop─┐    ┌─master_split_handle──┐
//  │     m2s_tx├───►│m2s_rx ───► tx_sm FIFO├───► pio
//  └───────────┘    └──────────────────────┘
//

/// Starts background task for master side that
/// - send data from slave to m2s channel.
/// - receive data from s2m channel and send it to slave.
pub async fn master_split_handle(p: SplitPeripherals, m2s_rx: M2sRx<'_>, s2m_tx: S2mTx<'_>) {
    let pio = Pio::new(p.pio, Irqs);
    let mut comm = Communicate::new(pio, p.data_pin).await;

    let mut buf = [0u8; MAX_DATA_SIZE];
    loop {
        match select(comm.recv_data::<MAX_DATA_SIZE>(&mut buf), m2s_rx.receive()).await {
            Either::First(_) => {
                let data = SlaveToMaster::from_bytes(&buf);

                let mut str = heapless::String::<512>::new();
                write!(
                    str,
                    "r:{:?}\n{:?}\n{:?}",
                    &buf[0..6],
                    &buf[7..13],
                    &buf[14..19]
                )
                .unwrap();
                DISPLAY.lock().await.as_mut().unwrap().draw_text(&str);

                s2m_tx.send(data).await;
            }
            Either::Second(send_data) => {
                comm.send_data::<MAX_DATA_SIZE>(send_data.to_bytes().as_slice())
                    .await;
            }
        }
    }
}

pub async fn slave_split_handle(p: SplitPeripherals, m2s_tx: M2sTx<'_>, s2m_rx: S2mRx<'_>) {
    let pio = Pio::new(p.pio, Irqs);
    let mut comm = Communicate::new(pio, p.data_pin).await;

    let mut buf = [0u8; MAX_DATA_SIZE];

    loop {
        match select(comm.recv_data::<MAX_DATA_SIZE>(&mut buf), s2m_rx.receive()).await {
            Either::First(_) => {
                // TODO: 入力値チェックをしたい(allocがないと無理？)
                let data = MasterToSlave::from_bytes(&buf);

                let mut str = heapless::String::<512>::new();
                write!(str, "recv_sl:\n{:?}", data).unwrap();
                DISPLAY.lock().await.as_mut().unwrap().draw_text(&str);

                m2s_tx.send(data).await;
            }
            Either::Second(send_data) => {
                let data = send_data.to_bytes();

                comm.send_data::<MAX_DATA_SIZE>(data.as_slice()).await;

                let mut str = heapless::String::<256>::new();
                write!(
                    str,
                    "s:{:?}\n{:?}\n{:?}",
                    &data[0..6],
                    &data[7..13],
                    &data[14..19]
                )
                .unwrap();
                DISPLAY.lock().await.as_mut().unwrap().draw_text(&str);
            }
        }
    }
}
