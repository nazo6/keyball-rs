use crate::{
    config::{LAYER_NUM, TAP_THRESHOLD},
    keyboard::keycode::{layer::LayerOp, special::Special, KeyAction, KeyCode},
};

use super::{
    all_pressed::{KeyStatusChangeType, KeyStatusUpdateEvent},
    MainloopState,
};

pub fn process_event(
    key_action: &KeyAction,
    event: &KeyStatusUpdateEvent,
    state: &mut MainloopState,
    layer_active: &mut [bool; LAYER_NUM],
    scroll_mode: &mut bool,
) {
    let Some(kc) = (match event.change_type {
        KeyStatusChangeType::Pressed => match key_action {
            KeyAction::Tap(kc) => Some(kc),
            _ => None,
        },
        KeyStatusChangeType::Pressing(duration) => match key_action {
            KeyAction::Tap(kc) => Some(kc),
            KeyAction::TapHold(_tkc, hkc) => {
                if duration > TAP_THRESHOLD {
                    Some(hkc)
                } else {
                    None
                }
            }
        },
        KeyStatusChangeType::Released(duration) => match key_action {
            KeyAction::Tap(kc) => Some(kc),
            KeyAction::TapHold(tkc, hkc) => {
                if duration > TAP_THRESHOLD {
                    Some(hkc)
                } else {
                    Some(tkc)
                }
            }
        },
    }) else {
        return;
    };

    match kc {
        KeyCode::Key(key) => {
            if let KeyStatusChangeType::Pressed = event.change_type {
                state.normal_key_pressed = true;
            }
            if state.keycodes_idx < 6 {
                state.keycodes[state.keycodes_idx] = *key as u8;
                state.keycodes_idx += 1;
            }
        }
        KeyCode::Media(key) => {
            state.media_key = Some(*key as u16);
        }
        KeyCode::Mouse(btn) => state.mouse_buttons |= btn.bits(),
        KeyCode::Modifier(mod_key) => {
            state.modifier |= mod_key.bits();
        }
        KeyCode::WithModifier(mod_key, key) => {
            if state.keycodes_idx < 6 {
                state.keycodes[state.keycodes_idx] = *key as u8;
                state.modifier |= mod_key.bits();
                state.keycodes_idx += 1;
            }
        }
        KeyCode::Layer(layer_op) => match event.change_type {
            KeyStatusChangeType::Released(_) => match layer_op {
                LayerOp::Move(l) => {
                    layer_active[*l] = false;
                }
                LayerOp::Toggle(l) => {
                    layer_active[*l] = !layer_active[*l];
                }
            },
            _ => match layer_op {
                LayerOp::Move(l) => {
                    layer_active[*l] = true;
                }
                _ => {}
            },
        },
        KeyCode::Special(special_op) => match event.change_type {
            KeyStatusChangeType::Released(_) => match special_op {
                Special::MoScrl => {
                    *scroll_mode = false;
                }
            },
            _ => match special_op {
                Special::MoScrl => {
                    *scroll_mode = true;
                }
            },
        },
    };
}
