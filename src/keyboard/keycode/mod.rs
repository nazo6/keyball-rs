use crate::constant::{COLS, ROWS};

pub mod key;
pub mod layer;
mod macros;
pub mod modifier;
pub mod mouse;
pub mod special;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum KeyCode {
    Modifier(modifier::Modifier),
    Mouse(mouse::Mouse),
    Key(key::Key),
    Special(special::Special),
    Layer(layer::Layer),
}

#[derive(PartialEq, Eq, Clone, Debug, Copy)]
pub enum KeyAction {
    None,
    Inherit,
    Normal(KeyCode),
    Tap(KeyCode),
    TapHold(KeyCode, KeyCode),
}

pub enum KeyCode2 {
    None,
    Inherit,
    Normal(key::Key),
    Modifier(modifier::Modifier),
    WithModifier(modifier::Modifier, key::Key),
    Mouse(mouse::Mouse),
    Special(special::Special),
    ModTap(modifier::Modifier, key::Key),
    Layer(layer::Layer),
}

pub const _____: KeyAction = KeyAction::Inherit;

pub type Layer = [[KeyAction; COLS * 2]; ROWS];
