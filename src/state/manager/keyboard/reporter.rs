use usbd_hid::descriptor::KeyboardReport;

pub struct KeyboardReportGenerator {
    empty_kb_sent: bool,
}

impl KeyboardReportGenerator {
    pub fn new() -> Self {
        Self {
            empty_kb_sent: false,
        }
    }

    pub fn gen(
        &mut self,
        keycodes: [u8; 6],
        modifier: u8,
        keycode_count: u8,
    ) -> Option<KeyboardReport> {
        if modifier == 0 && keycode_count == 0 {
            if !self.empty_kb_sent {
                self.empty_kb_sent = true;
                Some(KeyboardReport::default())
            } else {
                None
            }
        } else {
            self.empty_kb_sent = false;
            Some(KeyboardReport {
                keycodes,
                modifier,
                reserved: 0,
                leds: 0,
            })
        }
    }
}
