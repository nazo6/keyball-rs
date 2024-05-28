use embassy_time::Instant;
use usbd_hid::descriptor::MouseReport;

use crate::{
    config::{AUTO_MOUSE_DURATION, AUTO_MOUSE_LAYER, AUTO_MOUSE_THRESHOLD},
    keyboard::keycode::{special::Special, KeyCode},
    state::{
        common::{CommonLocalState, CommonState},
        pressed::{KeyStatusChangeType, KeyStatusUpdateEvent},
    },
};

use super::interface::LocalStateManager;

mod reporter;

/// Global mouse state
pub struct MouseState {
    scroll_mode: bool,
    reporter: reporter::MouseReportGenerator,
    aml: AmlState,
}

pub struct AmlState {
    start: Option<Instant>,
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            aml: AmlState { start: None },
            scroll_mode: false,
            reporter: reporter::MouseReportGenerator::new(),
        }
    }
}

/// Loop-local mouse state
pub struct MouseLocalState {
    pub mouse_event: (i8, i8),
    pub mouse_button: u8,
}

impl MouseLocalState {
    pub fn new(mouse_event: (i8, i8)) -> Self {
        Self {
            mouse_event,
            mouse_button: 0,
        }
    }
}

impl LocalStateManager for MouseLocalState {
    type GlobalState = MouseState;
    type Report = MouseReport;

    fn process_event(
        &mut self,
        _common_state: &mut CommonState,
        _common_local_state: &mut CommonLocalState,
        global_mouse_state: &mut MouseState,
        kc: &KeyCode,
        event: &KeyStatusUpdateEvent,
    ) {
        match kc {
            KeyCode::Mouse(btn) => self.mouse_button |= btn.bits(),
            KeyCode::Special(special_op) => match event.change_type {
                KeyStatusChangeType::Released(_) => match special_op {
                    Special::MoScrl => {
                        global_mouse_state.scroll_mode = false;
                    }
                },
                _ => match special_op {
                    Special::MoScrl => {
                        global_mouse_state.scroll_mode = true;
                    }
                },
            },
            _ => {}
        }
    }

    fn finalize(
        self,
        common_state: &mut CommonState,
        common_local_state: &mut CommonLocalState,
        global_mouse_state: &mut MouseState,
    ) -> Option<MouseReport> {
        if self.mouse_event.0.unsigned_abs() > AUTO_MOUSE_THRESHOLD
            || self.mouse_event.1.unsigned_abs() > AUTO_MOUSE_THRESHOLD
            || self.mouse_button != 0
            || global_mouse_state.scroll_mode
        {
            common_state.layer_active[AUTO_MOUSE_LAYER] = true;
            global_mouse_state.aml.start = Some(common_local_state.now);
        } else if let Some(start) = &global_mouse_state.aml.start {
            if common_local_state.now.duration_since(*start) > AUTO_MOUSE_DURATION
                || common_local_state.normal_key_pressed
            {
                common_state.layer_active[AUTO_MOUSE_LAYER] = false;
                global_mouse_state.aml.start = None;
            }
        };

        global_mouse_state.reporter.gen(
            self.mouse_event,
            self.mouse_button,
            global_mouse_state.scroll_mode,
        )
    }
}
