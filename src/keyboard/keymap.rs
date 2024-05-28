use crate::config::LAYER_NUM;

use super::keycode::key::*;
use super::keycode::layer::*;
use super::keycode::media::*;
use super::keycode::modifier::*;
use super::keycode::mouse::*;
use super::keycode::special::*;
use super::keycode::{Layer, *};

pub const L2ENTER: KeyDef = KeyDef::Key(KeyAction::TapHold(
    KeyCode::Key(Key::Enter),
    KeyCode::Layer(LayerOp::Move(2)),
));

#[rustfmt::skip]
const L0: LayerMap = [
    [ JZNHN , D1    , D2    , D3    , D4    , D5    , _____ , /**/ _____ , D6    , D7    , D8    , D9    ,D0    ,JBSLSH2],
    [  TAB  , Q     , W     , E     , R     , T     , _____ , /**/ _____ , Y     , U     , I     , O     ,P     , MINUS ],
    [  ESC  , A     , S     , D     , F     , G     , _____ , /**/ _____ , H     , J     , K     , L     ,SEMI  , JCOLN ],
    [ L_SHFT, Z     , X     , C     , V     , B     , JLBRC , /**/ JRBRC , N     , M     , COMM  , DOT   ,SLASH ,JBSLSH ],
    [ L_CTRL, L_GUI , MV(3) , TG(2) , L_ALT , SPACE , SPACE , /**/ BS    ,L2ENTER, _____ , _____ , _____ ,JCARET, JAT   ],
];

#[rustfmt::skip]
/// Auto mouse layer
const L1: LayerMap = [
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , _____ , _____ , _____ , _____ , _____ , _____ ],
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , LEFT  , DOWN  , UP    , RIGHT , _____ , _____ ],
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , _____ , M_L   ,MO_SCRL, M_R   , _____ , _____ ],
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , _____ , M_BCK , M_MID , M_FWD , _____ , _____ ],
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , _____ , _____ , _____ , _____ , _____ , _____ ],
];

#[rustfmt::skip]
/// Mouse layer
const L2: LayerMap = [
    [ _____ , F1    , F2    , F3    , F4    , F5    , _____ , /**/ _____ , F6    , F7    , F8    , F9    , F10   , F11   ],
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , LEFT  , DOWN  , UP    , RIGHT , _____ , F12   ],
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , _____ , M_L   ,MO_SCRL, M_R   , _____ , VOLUP ],
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , _____ , M_BCK , M_MID , M_FWD , _____ , VOLDN ],
    [ _____ , _____ , _____ , TG(2) , _____ , _____ , _____ , /**/ _____ , _____ , _____ , _____ , _____ , _____ , PRTSC ],
];

#[rustfmt::skip]
const L3: LayerMap = [
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , _____ , _____ , _____ , _____ , _____ , _____ ],
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , _____ , _____ , _____ , _____ , _____ , _____ ],
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , _____ , M_L   , M_MID , M_R   , _____ , _____ ],
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , _____ , _____ , _____ , _____ , _____ , _____ ],
    [ _____ , _____ , _____ , TG(0) , _____ , _____ , _____ , /**/ _____ , _____ , _____ , _____ , _____ , _____ , _____ ],
];

pub const KEYMAP: [Layer; LAYER_NUM] = [
    Layer {
        map: L0,
        arrowball: false,
    },
    Layer {
        map: L1,
        arrowball: false,
    },
    Layer {
        map: L2,
        arrowball: false,
    },
    Layer {
        map: L3,
        arrowball: true,
    },
];
