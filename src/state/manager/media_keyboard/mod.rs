use usbd_hid::descriptor::MediaKeyboardReport;

use crate::{
    keyboard::keycode::KeyCode,
    state::{
        common::{CommonLocalState, CommonState},
        pressed::KeyStatusUpdateEvent,
    },
};

use super::interface::LocalStateManager;

mod reporter;

pub struct MediaKeyboardState {
    reporter: reporter::MediaKeyboardReportGenerator,
}

impl MediaKeyboardState {
    pub fn new() -> Self {
        Self {
            reporter: reporter::MediaKeyboardReportGenerator::new(),
        }
    }
}

pub struct MediaKeyboardLocalState {
    pub media_key: Option<u16>,
}

impl MediaKeyboardLocalState {
    pub fn new() -> Self {
        Self { media_key: None }
    }
}

impl LocalStateManager for MediaKeyboardLocalState {
    type GlobalState = MediaKeyboardState;

    type Report = MediaKeyboardReport;

    fn process_event(
        &mut self,
        _common_state: &mut CommonState,
        _common_local_state: &mut CommonLocalState,
        _global_state: &mut Self::GlobalState,
        kc: &crate::keyboard::keycode::KeyCode,
        _event: &KeyStatusUpdateEvent,
    ) {
        match kc {
            KeyCode::Media(key) => {
                self.media_key = Some(*key as u16);
            }
            _ => {}
        };
    }

    fn report(
        self,
        common_state: &CommonState,
        common_local_state: &CommonLocalState,
        global_state: &mut Self::GlobalState,
    ) -> Option<Self::Report> {
        global_state.reporter.gen(self.media_key)
    }
}
