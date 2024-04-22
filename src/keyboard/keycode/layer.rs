#![allow(non_snake_case)]

use crate::constant::LAYER_NUM;

use super::{KeyAction, KeyCode};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Layer {
    Move(usize),
    Toggle(usize),
}

pub const fn MOVE(n: usize) -> KeyAction {
    assert!(n < LAYER_NUM);
    KeyAction::Normal(KeyCode::Layer(Layer::Move(n)))
}

pub const fn TG(n: usize) -> KeyAction {
    assert!(n < LAYER_NUM);
    KeyAction::Normal(KeyCode::Layer(Layer::Toggle(n)))
}
