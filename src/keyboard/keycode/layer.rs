use super::Keycode;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Layer {
    Move(usize),
    Toggle(usize),
}

#[allow(non_snake_case)]
pub const fn MOVE(n: usize) -> Keycode {
    Keycode::Layer(Layer::Move(n))
}
