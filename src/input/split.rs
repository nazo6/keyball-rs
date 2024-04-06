use embassy_futures::select::{select, Either};
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{Common, Config, InterruptHandler, Pin, Pio, StateMachine};
use embassy_rp::{bind_interrupts, Peripheral};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;

use super::SplitInputPeripherals;

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

pub enum MasterToSlave {
    Ping,
    Led,
}

pub enum SlaveToMaster {
    Pong,
    KeyChange {
        change_type: KeyChangeType,
        row: u8,
        col: u8,
    },
    Mouse {
        dx: u8,
        dy: u8,
    },
}

pub enum KeyChangeType {
    Pressed,
    Released,
}

fn rx_init<'a>(
    common: &mut Common<'a, PIO0>,
    sm: &mut StateMachine<'a, PIO0, 0>,
    data_pin: &Pin<'a, PIO0>,
) {
    let prg = pio_proc::pio_asm!(
        "wait 0 pin 0",
        "set  x 7",
        "in   pins 1",
        "jmp  x-- 2",
        "push block",
    );
    let mut cfg = Config::default();
    cfg.use_program(&common.load_program(&prg.program), &[]);
    cfg.set_out_pins(&[data_pin]);
    cfg.set_set_pins(&[data_pin]);
    cfg.clock_divider = (U56F8!(125_000_000) / 20 / 200).to_fixed();
    sm.set_config(&cfg);
}

fn tx_init<'a>(
    common: &mut Common<'a, PIO0>,
    sm: &mut StateMachine<'a, PIO0, 1>,
    data_pin: &Pin<'a, PIO0>,
) {
    let prg = pio_proc::pio_asm!(
        "pull block",
        "set x 7",
        "out pindirs 1",
        "jmp x-- 2",
        "push",
    );
    let mut cfg = Config::default();
    cfg.use_program(&common.load_program(&prg.program), &[]);
    cfg.set_out_pins(&[&data_pin]);
    cfg.set_set_pins(&[&data_pin]);
    cfg.clock_divider = (U56F8!(125_000_000) / 20 / 200).to_fixed();
    cfg.shift_out.auto_fill = true;
    sm.set_config(&cfg);
}

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
    let Pio {
        mut common,
        mut sm0,
        mut sm1,
        ..
    } = Pio::new(p.pio, Irqs);

    let out_pin = common.make_pio_pin(p.data_pin);
    rx_init(&mut common, &mut sm0, &out_pin);
    tx_init(&mut common, &mut sm1, &out_pin);

    let rx = sm0.rx();
    let mut buf = [0u32; 3];
    let mut dma = p.dma.into_ref();
    loop {
        match select(rx.dma_pull(dma.reborrow(), &mut buf), m2s_rx.receive()).await {
            Either::First(_) => {
                // TODO: Put received data into a s2m_tx
            }
            Either::Second(send_data) => {
                // TODO: Send data to slave
            }
        }
    }
}

pub async fn slave_split_handle(p: SplitInputPeripherals, m2s_tx: M2sTx<'_>, s2m_rx: S2mRx<'_>) {
    let Pio {
        mut common,
        mut sm0,
        mut sm1,
        ..
    } = Pio::new(p.pio, Irqs);

    let out_pin = common.make_pio_pin(p.data_pin);
    rx_init(&mut common, &mut sm0, &out_pin);
    tx_init(&mut common, &mut sm1, &out_pin);

    let rx = sm0.rx();
    let mut buf = [0u32; 3];
    let mut dma = p.dma.into_ref();
    loop {
        match select(rx.dma_pull(dma.reborrow(), &mut buf), s2m_rx.receive()).await {
            Either::First(_) => {
                // TODO: Put received data into a m2s_tx
            }
            Either::Second(send_data) => {
                // TODO: Send data to master
            }
        }
    }
}
