use super::{macros::normal, KeyCode};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Mouse {
    Left = 0b0000_0001,
    Right = 0b0000_0010,
    Middle = 0b0000_0100,
    Back = 0b0000_1000,
    Forward = 0b0001_0000,
}

normal!(M_L, Mouse, Left);
normal!(M_R, Mouse, Right);
normal!(M_MID, Mouse, Middle);
normal!(M_BCK, Mouse, Back);
normal!(M_FWD, Mouse, Forward);
