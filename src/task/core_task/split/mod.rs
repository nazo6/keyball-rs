use crate::constant::SPLIT_CHANNEL_SIZE;
use crate::driver::split::Communicate;
use embassy_futures::select::{select, Either};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};

use super::SplitPeripherals;

mod data;
pub use data::*;

pub type S2mChannel = Channel<ThreadModeRawMutex, SlaveToMaster, SPLIT_CHANNEL_SIZE>;
pub type S2mRx<'a> = Receiver<'a, ThreadModeRawMutex, SlaveToMaster, SPLIT_CHANNEL_SIZE>;
pub type S2mTx<'a> = Sender<'a, ThreadModeRawMutex, SlaveToMaster, SPLIT_CHANNEL_SIZE>;

pub type M2sChannel = Channel<ThreadModeRawMutex, MasterToSlave, SPLIT_CHANNEL_SIZE>;
pub type M2sRx<'a> = Receiver<'a, ThreadModeRawMutex, MasterToSlave, SPLIT_CHANNEL_SIZE>;
pub type M2sTx<'a> = Sender<'a, ThreadModeRawMutex, MasterToSlave, SPLIT_CHANNEL_SIZE>;

/// Starts background task for master side that
/// - send data from slave to m2s channel.
/// - receive data from s2m channel and send it to slave.
pub async fn master_split_handle(p: SplitPeripherals, m2s_rx: M2sRx<'_>, s2m_tx: S2mTx<'_>) {
    let mut comm = Communicate::new(p).await;

    let mut buf = [0u8; MAX_DATA_SIZE];
    loop {
        match select(comm.recv_data::<MAX_DATA_SIZE>(&mut buf), m2s_rx.receive()).await {
            Either::First(_) => {
                let data = SlaveToMaster::from_bytes(&buf);

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
    let mut comm = Communicate::new(p).await;

    let mut buf = [0u8; MAX_DATA_SIZE];

    loop {
        match select(comm.recv_data::<MAX_DATA_SIZE>(&mut buf), s2m_rx.receive()).await {
            Either::First(_) => {
                // TODO: 入力値チェックをしたい(allocがないと無理？)
                let data = MasterToSlave::from_bytes(&buf);

                m2s_tx.send(data).await;
            }
            Either::Second(send_data) => {
                let data = send_data.to_bytes();

                comm.send_data::<MAX_DATA_SIZE>(data.as_slice()).await;
            }
        }
    }
}
