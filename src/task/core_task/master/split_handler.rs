use embassy_futures::select::{select, Either};
use postcard::{from_bytes, to_slice};

use crate::{device::peripherals::SplitPeripherals, driver::split::Communicate};

use super::super::split::*;

/// Starts background task for master side that
/// - send data from slave to m2s channel.
/// - receive data from s2m channel and send it to slave.
pub async fn start(p: SplitPeripherals, m2s_rx: M2sRx<'_>, s2m_tx: S2mTx<'_>) {
    let mut comm = Communicate::new(p).await;

    let mut recv_buf = [0u8; MAX_DATA_SIZE];
    let mut send_buf = [0u8; MAX_DATA_SIZE];

    loop {
        match select(
            comm.recv_data::<MAX_DATA_SIZE>(&mut recv_buf),
            m2s_rx.receive(),
        )
        .await
        {
            Either::First(_) => {
                if let Ok(data) = from_bytes(&recv_buf) {
                    let _ = s2m_tx.send(data).await;
                }
            }
            Either::Second(send_data) => {
                if let Ok(bytes) = to_slice(&send_data, &mut send_buf) {
                    comm.send_data::<MAX_DATA_SIZE>(bytes).await;
                }
            }
        }
    }
}
