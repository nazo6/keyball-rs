use embassy_futures::select::{select, Either};

use crate::{device::peripherals::SplitPeripherals, driver::split::Communicate};

use super::super::split::*;

/// Starts background task for master side that
/// - send data from slave to m2s channel.
/// - receive data from s2m channel and send it to slave.
pub async fn start(p: SplitPeripherals, m2s_rx: M2sRx<'_>, s2m_tx: S2mTx<'_>) {
    let mut comm = Communicate::new(p).await;

    let mut buf = [0u8; MAX_DATA_SIZE];

    loop {
        match select(comm.recv_data::<MAX_DATA_SIZE>(&mut buf), m2s_rx.receive()).await {
            Either::First(_) => {
                let data = SlaveToMaster::from_bytes(&buf);
                let _ = s2m_tx.send(data).await;
            }
            Either::Second(send_data) => {
                comm.send_data::<MAX_DATA_SIZE>(send_data.to_bytes().as_slice())
                    .await;
            }
        }
    }
}
