use core::fmt::Write;

use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};

use crate::{
    device::peripherals::DisplayPeripherals,
    driver::{display::Oled, keyboard::Hand},
};

pub static DISPLAY: GlobalDisplay = GlobalDisplay::new();

macro_rules! update_display {
    ($self:expr, $str:expr, $x:literal, $y:literal) => {
        $self
            .inner
            .lock()
            .await
            .as_mut()
            .unwrap()
            .update_text($str, Oled::calculate_point($x, $y))
            .await;
    };
}

pub struct GlobalDisplay {
    pub inner: Mutex<ThreadModeRawMutex, Option<Oled<'static>>>,
}

impl GlobalDisplay {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }
    pub async fn init(&self, p: DisplayPeripherals) {
        self.inner.lock().await.replace(Oled::new(p));
    }

    pub fn try_draw_text(&self, str: &str) {
        if let Ok(mut display) = self.inner.try_lock() {
            display.as_mut().unwrap().draw_text_blocking(str);
        }
    }

    pub fn try_set_message(&self, str: &str) {
        if let Ok(mut display) = self.inner.try_lock() {
            display
                .as_mut()
                .unwrap()
                .update_text_blocking(str, Oled::calculate_point(1, 3));
        }
    }

    pub async fn clear(&self) {
        self.inner.lock().await.as_mut().unwrap().clear().await;
    }

    pub async fn set_message(&self, str: &str) {
        update_display!(self, str, 1, 3);
    }

    pub async fn set_master(&self, master: bool) {
        update_display!(self, if master { "M" } else { "S" }, 1, 1);
    }

    pub async fn set_mouse(&self, mouse: bool) {
        update_display!(self, if mouse { "m" } else { "x" }, 2, 1);
    }

    pub async fn set_hand(&self, hand: Hand) {
        update_display!(self, if hand == Hand::Left { "L" } else { "R" }, 3, 1);
    }

    pub async fn set_highest_layer(&self, layer: u8) {
        let mut str = heapless::String::<2>::new();
        write!(str, "{:1}", layer).unwrap();
        update_display!(self, &str, 5, 1);
    }

    pub async fn set_mouse_pos(&self, x: i8, y: i8) {
        let mut str = heapless::String::<32>::new();
        write!(str, "[{:3},{:3}]", x, y).unwrap();
        update_display!(self, &str, 8, 1);
    }
}
