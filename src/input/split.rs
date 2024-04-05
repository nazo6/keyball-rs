use core::fmt::Write;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::PIO0;
use embassy_rp::pio::{Config, InterruptHandler, Pio};
use embassy_time::Timer;
use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;

use crate::DISPLAY;

use super::SplitInputPeripherals;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

pub async fn start(p: SplitInputPeripherals) {
    let pio = p.pio;

    let Pio {
        mut common,
        mut sm0,
        mut sm1,
        ..
    } = Pio::new(pio, Irqs);

    let out_pin = common.make_pio_pin(p.data_pin);

    let rx_task = {
        let prg = pio_proc::pio_asm!(
            "wait 0 pin 0",
            "set  x 7",
            "in   pins 1",
            "jmp  x-- 2",
            "push block",
        );
        let mut cfg = Config::default();
        cfg.use_program(&common.load_program(&prg.program), &[]);
        cfg.set_out_pins(&[&out_pin]);
        cfg.set_set_pins(&[&out_pin]);
        cfg.clock_divider = (U56F8!(125_000_000) / 20 / 200).to_fixed();
        sm0.set_config(&cfg);

        async {
            let sm0_rx = sm0.rx();
            loop {
                let data = sm0_rx.wait_pull().await;
                let mut str = heapless::String::<100>::new();
                write!(&mut str, "receive: {}", data).unwrap();
                DISPLAY.lock().await.as_mut().unwrap().draw_text(&str);
            }
        }
    };

    let tx_task = {
        let prg = pio_proc::pio_asm!(
            "pull block",
            "set x 7",
            "out pindirs 1",
            "jmp x-- 2",
            "push",
        );
        let mut cfg = Config::default();
        cfg.use_program(&common.load_program(&prg.program), &[]);
        cfg.set_out_pins(&[&out_pin]);
        cfg.set_set_pins(&[&out_pin]);
        cfg.clock_divider = (U56F8!(125_000_000) / 20 / 200).to_fixed();
        cfg.shift_out.auto_fill = true;
        sm1.set_config(&cfg);
        async {
            let sm1_tx = sm1.tx();
            loop {
                sm1_tx.push(100);
                Timer::after_millis(100).await;
            }
        }
    };

    join(rx_task, tx_task).await;
}
