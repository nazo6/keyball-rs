#![allow(non_snake_case)]

use crate::{constant::LAYER_NUM, keyboard::keycode::KeyDef};

use super::{KeyAction, KeyCode};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum LayerOp {
    Move(usize),
    Toggle(usize),
}

pub const fn MOVE(n: usize) -> KeyDef {
    assert!(n < LAYER_NUM);
    KeyDef::Key(KeyAction::Tap(KeyCode::Layer(LayerOp::Move(n))))
}

pub const fn TG(n: usize) -> KeyDef {
    assert!(n < LAYER_NUM);
    KeyDef::Key(KeyAction::Tap(KeyCode::Layer(LayerOp::Toggle(n))))
}
