#![allow(non_snake_case)]

use crate::constant::LAYER_NUM;

use super::Keycode;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Layer {
    Move(usize),
    Toggle(usize),
}

pub const fn MOVE(n: usize) -> Keycode {
    assert!(n < LAYER_NUM);
    Keycode::Layer(Layer::Move(n))
}

pub const fn TG(n: usize) -> Keycode {
    assert!(n < LAYER_NUM);
    Keycode::Layer(Layer::Toggle(n))
}
