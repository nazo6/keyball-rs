use core::fmt::Write;

use embassy_futures::select::{select, Either};
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use rkyv::ser::serializers::BufferSerializer;
use rkyv::ser::Serializer;
use rkyv::{AlignedBytes, Archive, Deserialize, Serialize};

use crate::DISPLAY;

use self::bit_layer::Communicate;

use super::SplitInputPeripherals;

mod bit_layer;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

const SPLIT_CHANNEL_SIZE: usize = 10;
pub type S2mChannel = Channel<ThreadModeRawMutex, SlaveToMaster, SPLIT_CHANNEL_SIZE>;
pub type S2mRx<'a> = Receiver<'a, ThreadModeRawMutex, SlaveToMaster, SPLIT_CHANNEL_SIZE>;
pub type S2mTx<'a> = Sender<'a, ThreadModeRawMutex, SlaveToMaster, SPLIT_CHANNEL_SIZE>;

pub type M2sChannel = Channel<ThreadModeRawMutex, MasterToSlave, SPLIT_CHANNEL_SIZE>;
pub type M2sRx<'a> = Receiver<'a, ThreadModeRawMutex, MasterToSlave, SPLIT_CHANNEL_SIZE>;
pub type M2sTx<'a> = Sender<'a, ThreadModeRawMutex, MasterToSlave, SPLIT_CHANNEL_SIZE>;

#[derive(Archive, Deserialize, Serialize, Debug)]
// #[archive(check_bytes)]
pub enum MasterToSlave {
    Led,
    Message(u8),
}

#[derive(Archive, Deserialize, Serialize, Debug)]
// #[archive(check_bytes)]
pub enum SlaveToMaster {
    KeyChange {
        change_type: KeyChangeType,
        row: u8,
        col: u8,
    },
    Mouse {
        dx: u8,
        dy: u8,
    },
    Message(u8),
}

#[derive(Archive, Deserialize, Serialize, Debug)]
// #[archive(check_bytes)]
pub enum KeyChangeType {
    Pressed,
    Released,
}

const DATA_SIZE: usize = 1;

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
pub async fn master_split_handle(p: SplitInputPeripherals, m2s_rx: M2sRx<'_>, s2m_tx: S2mTx<'_>) {
    let pio = Pio::new(p.pio, Irqs);
    let mut comm = Communicate::new(pio, p.data_pin);

    let mut buf = [0u8; DATA_SIZE];

    loop {
        match select(comm.recv_data::<DATA_SIZE>(&mut buf), m2s_rx.receive()).await {
            Either::First(_) => {}
            Either::Second(send_data) => {
                //
            }
        }
    }
}

pub async fn slave_split_handle(p: SplitInputPeripherals, m2s_tx: M2sTx<'_>, s2m_rx: S2mRx<'_>) {
    let pio = Pio::new(p.pio, Irqs);
    let mut comm = Communicate::new(pio, p.data_pin);

    let mut buf = [0u8; DATA_SIZE];

    let mut test_data = [0u8; DATA_SIZE];

    let mut cnt = 0;

    loop {
        match select(
            comm.recv_data::<DATA_SIZE>(&mut buf),
            embassy_time::Timer::after_millis(500),
        )
        .await
        {
            Either::First(_) => {
                cnt += 1;
            }
            Either::Second(_) => {
                comm.send_data::<DATA_SIZE>(&test_data).await;
                test_data[0] += 1;
            }
        }
    }
}
