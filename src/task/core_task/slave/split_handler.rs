use embassy_futures::select::{select, Either};
use postcard::{from_bytes, to_slice};

use crate::{device::peripherals::SplitPeripherals, driver::split::Communicate};

use super::super::split::*;

pub async fn start(p: SplitPeripherals, m2s_tx: M2sTx<'_>, s2m_rx: S2mRx<'_>) {
    let mut comm = Communicate::new(p).await;

    let mut recv_buf = [0u8; MAX_DATA_SIZE];
    let mut send_buf = [0u8; MAX_DATA_SIZE];

    loop {
        match select(
            comm.recv_data::<MAX_DATA_SIZE>(&mut recv_buf),
            s2m_rx.receive(),
        )
        .await
        {
            Either::First(_) => {
                if let Ok(data) = from_bytes(&recv_buf) {
                    let _ = m2s_tx.send(data).await;
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
