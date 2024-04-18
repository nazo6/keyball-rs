use super::Keycode;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Mouse {
    Left = 0b0000_0001,
    Right = 0b0000_0010,
    Middle = 0b0000_0100,
    Back = 0b0000_1000,
    Forward = 0b0001_0000,
}

pub const M_L: Keycode = Keycode::Mouse(Mouse::Left);
pub const M_R: Keycode = Keycode::Mouse(Mouse::Right);
pub const M_MID: Keycode = Keycode::Mouse(Mouse::Middle);
pub const M_BCK: Keycode = Keycode::Mouse(Mouse::Back);
pub const M_FWD: Keycode = Keycode::Mouse(Mouse::Forward);
