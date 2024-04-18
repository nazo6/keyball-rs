pub mod key;
pub mod layer;
mod macros;
pub mod modifier;
pub mod mouse;
pub mod special;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Keycode {
    Modifier(modifier::Modifier),
    Mouse(mouse::Mouse),
    Key(key::Key),
    Special(special::Special),
    Layer(layer::Layer),
}

pub const ____: Keycode = Keycode::Special(special::Special::Inherit);
