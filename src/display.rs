use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};

use crate::{device::peripherals::DisplayPeripherals, driver::display::Oled};

type DisplayType = Mutex<ThreadModeRawMutex, Option<Oled<'static>>>;
pub static DISPLAY: DisplayType = Mutex::new(None);

pub async fn init_display(p: DisplayPeripherals) {
    let mut display = Oled::new(p);

    display.draw_text("Initialising...");

    DISPLAY.lock().await.replace(display);
}
