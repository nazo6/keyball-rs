use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    ptr::{read_volatile, write_volatile},
};

use embassy_time::Timer;

const BOOTLOADER_MAGIC: u32 = 0xDEADBEEF;

#[link_section = ".uninit.FLAG"]
static mut FLAG: UnsafeCell<MaybeUninit<u32>> = UnsafeCell::new(MaybeUninit::uninit());

pub async fn check_double_tap(timeout: u64) {
    unsafe {
        if read_volatile(FLAG.get().cast::<u32>()) == BOOTLOADER_MAGIC {
            write_volatile(FLAG.get().cast(), 0);

            embassy_rp::rom_data::reset_to_usb_boot(0, 0);
        }

        write_volatile(FLAG.get().cast(), BOOTLOADER_MAGIC);

        Timer::after_millis(timeout).await;

        write_volatile(FLAG.get().cast(), 0);
    }
}
