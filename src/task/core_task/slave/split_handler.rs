use embassy_futures::select::{select, Either};

use crate::{device::peripherals::SplitPeripherals, driver::split::Communicate};

use super::super::split::*;

pub async fn start(p: SplitPeripherals, m2s_tx: M2sTx<'_>, s2m_rx: S2mRx<'_>) {
    let mut comm = Communicate::new(p).await;

    let mut buf = [0u8; MAX_DATA_SIZE];

    loop {
        match select(comm.recv_data::<MAX_DATA_SIZE>(&mut buf), s2m_rx.receive()).await {
            Either::First(_) => {
                let data = MasterToSlave::from_bytes(&buf);
                let _ = m2s_tx.try_send(data);
            }
            Either::Second(send_data) => {
                let data = send_data.to_bytes();
                comm.send_data::<MAX_DATA_SIZE>(data.as_slice()).await;
            }
        }
    }
}
