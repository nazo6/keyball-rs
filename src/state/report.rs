use usbd_hid::descriptor::{KeyboardReport, MouseReport};

pub struct StateReport {
    pub keyboard_report: Option<KeyboardReport>,
    pub mouse_report: Option<MouseReport>,
    pub highest_layer: u8,
}
