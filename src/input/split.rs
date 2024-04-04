use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::{PIN_1, PIO0};
use embassy_rp::pio::{Config, InterruptHandler, Pio};
use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

pub struct SplitInputPeripherals {
    pub pio: PIO0,
    pub data_pin: PIN_1,
}

async fn start(p: SplitInputPeripherals) {
    let pio = p.pio;

    let Pio {
        mut common,
        mut irq0,
        mut sm0,
        mut sm1,
        ..
    } = Pio::new(pio, Irqs);

    let out_pin = common.make_pio_pin(p.data_pin);

    let rx_task = {
        let prg = pio_proc::pio_asm!(
            ".origin 0",
            ".wrap_target",
            "wait 0     pin 0",
            "set  x     31",
            "in   x     1",
            "jmp  x--   2",
            "push block",
            "irq  wait  0"
            ".wrap",
        );
        let mut cfg = Config::default();
        cfg.use_program(&common.load_program(&prg.program), &[]);
        cfg.set_out_pins(&[&out_pin]);
        cfg.set_set_pins(&[&out_pin]);
        cfg.clock_divider = (U56F8!(125_000_000) / 20 / 200).to_fixed();
        cfg.shift_out.auto_fill = true;
        sm0.set_config(&cfg);

        async {
            let sm0_rx = sm0.rx();
            loop {
                irq0.wait().await;
                let data = sm0_rx.wait_pull().await;
            }
        }
    };

    let tx_task = {
        let prg = pio_proc::pio_asm!(
            ".origin 0",
            ".wrap_target",
            "pull block",
            "set x     31",
            "out x     pins, 1",
        );
        let mut cfg = Config::default();
        cfg.use_program(&common.load_program(&prg.program), &[]);
        cfg.set_out_pins(&[&out_pin]);
        cfg.set_set_pins(&[&out_pin]);
        cfg.clock_divider = (U56F8!(125_000_000) / 20 / 200).to_fixed();
        cfg.shift_out.auto_fill = true;
        sm1.set_config(&cfg);
        async {
            loop {
                //
            }
        }
    };

    join(rx_task, tx_task).await;
}
