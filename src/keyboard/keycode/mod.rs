use crate::constant::{COLS, ROWS};

pub mod key;
pub mod layer;
pub mod macros;
pub mod modifier;
pub mod mouse;
pub mod special;

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum KeyCode {
    Key(key::Key),
    Mouse(mouse::Mouse),
    Modifier(modifier::Modifier),
    WithModifier(modifier::Modifier, key::Key),
    Layer(layer::LayerOp),
    Special(special::Special),
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum KeyAction {
    Tap(KeyCode),
    TapHold(KeyCode, KeyCode),
}

#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum KeyDef {
    None,
    Inherit,
    Key(KeyAction),
}

pub const _____: KeyDef = KeyDef::Inherit;
pub const XXXXX: KeyDef = KeyDef::None;

pub type Layer = [[KeyDef; COLS * 2]; ROWS];
