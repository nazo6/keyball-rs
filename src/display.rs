use core::fmt::Write as _;

use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};

use crate::{
    device::peripherals::DisplayPeripherals,
    driver::{display::Oled, keyboard::Hand},
};

pub static DISPLAY: GlobalDisplay = GlobalDisplay::new();

pub struct DisplayState {
    pub master: Option<bool>,
    pub message: heapless::String<512>,
    pub mouse_available: Option<bool>,
    pub keyboard: [u8; 6],
    pub mouse: (i8, i8),
    pub tick: bool,
    pub hand: Option<Hand>,
}

pub struct GlobalDisplay {
    pub inner: Mutex<ThreadModeRawMutex, Option<Oled<'static>>>,
    pub state: Mutex<ThreadModeRawMutex, DisplayState>,
}

impl GlobalDisplay {
    pub const fn new() -> Self {
        Self {
            inner: Mutex::new(None),
            state: Mutex::new(DisplayState {
                master: None,
                message: heapless::String::new(),
                mouse_available: None,
                keyboard: [0; 6],
                mouse: (0, 0),
                tick: false,
                hand: None,
            }),
        }
    }
    pub async fn init(&self, p: DisplayPeripherals) {
        let mut display = Oled::new(p);
        display.draw_text("Initialising...");
        self.inner.lock().await.replace(display);
    }

    pub fn try_draw_text(&self, str: &str) {
        if let Ok(mut display) = self.inner.try_lock() {
            display.as_mut().unwrap().draw_text(str);
        }
    }

    /// Panic: if display is not initialized
    pub async fn draw_text(&self, str: &str) {
        self.inner.lock().await.as_mut().unwrap().draw_text(str);
    }

    // TODO: 多分効率がかなり悪い
    async fn update(&self, tick: Option<(&str)>) {
        let mut str = heapless::String::<512>::new();

        let mut state = self.state.lock().await;

        if let Some(tick_s) = tick {
            write!(str, "{}", tick_s).unwrap();
            state.tick = !state.tick;
        } else {
            write!(str, " ").unwrap();
        }

        if state.tick {
            write!(str, "/").unwrap();
        } else {
            write!(str, "\\").unwrap();
        }

        if let Some(master) = state.master {
            if master {
                write!(str, "m").unwrap();
            } else {
                write!(str, "s").unwrap();
            }
        } else {
            write!(str, "?").unwrap();
        }

        if let Some(mouse) = state.mouse_available {
            if mouse {
                write!(str, "m").unwrap();
            } else {
                write!(str, "x").unwrap();
            }
        } else {
            write!(str, "?").unwrap();
        }

        if let Some(hand) = &state.hand {
            match hand {
                Hand::Right => write!(str, "r").unwrap(),
                Hand::Left => write!(str, "l").unwrap(),
            }
        } else {
            write!(str, "?").unwrap();
        }

        write!(str, ":{},{}", state.mouse.0, state.mouse.1).unwrap();

        if state.master.unwrap_or(false) {
            for key in state.keyboard.iter() {
                if *key != 0 {
                    write!(str, "{},", key).unwrap();
                }
            }
        }

        writeln!(str).unwrap();
        write!(str, "{}", state.message).unwrap();

        self.inner.lock().await.as_mut().unwrap().draw_text(&str);
    }

    pub async fn set_message(&self, str: &str) {
        self.state.lock().await.message.clear();
        self.state.lock().await.message.push_str(str).unwrap();
        self.update(Some("e")).await;
    }

    pub async fn set_master(&self, master: bool) {
        self.state.lock().await.master.replace(master);
        self.update(Some("a")).await;
    }

    pub async fn set_mouse(&self, mouse: bool) {
        self.state.lock().await.mouse_available.replace(mouse);
        self.update(Some("o")).await;
    }

    pub async fn set_keyboard(&self, keys: [u8; 6]) {
        self.state.lock().await.keyboard = keys;
        self.update(Some("k")).await;
    }

    pub async fn set_mouse_pos(&self, x: i8, y: i8) {
        self.state.lock().await.mouse = (x, y);
        self.update(Some("p")).await;
    }

    pub async fn set_hand(&self, hand: Hand) {
        self.state.lock().await.hand.replace(hand);
        self.update(Some("h")).await;
    }
}
