use super::keycode::key::*;
use super::keycode::modifier::*;
use super::keycode::*;
use crate::constant::{COLS, ROWS};

#[rustfmt::skip]
pub const KEYMAP: [[Keycode; COLS * 2]; ROWS] = [
    [____  , D1   , D2  , D3  , D4   , D5   , ____ , ____, D6   , D7  , D8  , D9  , D0    , JBSLSH2],
    [TAB   , Q    , W   , E   , R    , T    , ____ , ____, Y    , U   , I   , O   , P     , MINUS  ],
    [ESC   , A    , S   , D   , F    , G    , ____ , ____, H    , J   , K   , L   , SEMI  , JCOLN  ],
    [L_SHFT, Z    , X   , C   , V    , B    , ____ , ____, N    , M   , COMM, DOT , SLASH , JBSLSH ],
    [L_CTRL, L_GUI, ____, ____, L_ALT, SPACE, SPACE, BS  , ENTER, ____, ____, ____, JCARET, JAT    ],
];
