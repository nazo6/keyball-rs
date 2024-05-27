use crate::{
    keyboard::keycode::KeyCode,
    state::common::{CommonLocalState, CommonState},
};

use super::super::pressed::KeyStatusUpdateEvent;

pub(crate) trait LocalStateManager {
    type GlobalState;
    type Report;

    fn process_event(
        &mut self,
        common_state: &mut CommonState,
        common_local_state: &mut CommonLocalState,
        global_state: &mut Self::GlobalState,
        kc: &KeyCode,
        event: &KeyStatusUpdateEvent,
    );

    fn finalize(
        self,
        common_state: &mut CommonState,
        common_local_state: &mut CommonLocalState,
        global_state: &mut Self::GlobalState,
    ) -> Option<Self::Report>;
}
