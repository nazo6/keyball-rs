use rktk::keymanager::keycode::*;
use rktk::keymanager::keycode::{
    key::*, layer::*, media::*, modifier::*, mouse::*, special::*, utils::*,
};
use rktk::keymanager::keymap::TapDanceDefinition;
use rktk::keymap_config::{Keymap, Layer, LayerMap};

const L2ENTER: KeyAction = KeyAction::TapHold(
    KeyCode::Key(Key::Enter),
    KeyCode::Layer(LayerOp::Momentary(2)),
);

const L2SPC: KeyAction = KeyAction::TapHold(
    KeyCode::Key(Key::Enter),
    KeyCode::Layer(LayerOp::Momentary(2)),
);

const L3SPC: KeyAction = KeyAction::TapHold(
    KeyCode::Key(Key::Enter),
    KeyCode::Layer(LayerOp::Momentary(3)),
);

const L4GRV: KeyAction = KeyAction::TapHold(
    KeyCode::Key(Key::Grave),
    KeyCode::Layer(LayerOp::Momentary(4)),
);

const FL_CLR: KeyAction = KeyAction::Normal(KeyCode::Special(Special::FlashClear));

#[rustfmt::skip]
const L0: LayerMap = [
    [ L4GRV , D1    , D2    , D3    , D4    , D5    , _____ , /**/ _____ , D6    , D7    , D8    , D9    , D0   , EQUAL ],
    [  TAB  , Q     , W     , E     , R     , T     , _____ , /**/ _____ , Y     , U     , I     , O     , P    , MINUS],
    [  ESC  , A     , S     , D     , F     , G     , _____ , /**/ _____ , H     , J     , K     , L     , SCLN , QUOTE],
    [ L_SHFT, Z     , X     , C     , V     , B     , LBRC  , /**/ TD(0) , N     , M     , COMM  , DOT   , SLASH, BSLSH],
    [ L_CTRL, L_GUI , TG(2) , L_ALT , L3SPC , L2SPC , SPACE , /**/ BS    ,L2ENTER, _____ , _____ , _____ ,R_SHFT,R_CTRL],
];

#[rustfmt::skip]
/// Auto mouse layer
const L1: LayerMap = [
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , _____ , _____ , _____ , _____ , _____ , _____ ],
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , _____ , _____ , _____ , _____ , _____ , _____ ],
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____,AML_RESET,M_LEFT,MO_SCRL,M_RIGHT, _____ , _____ ],
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , _____ ,M_BACK,M_MIDDLE,M_FORWARD,_____, _____ ],
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , _____ , _____ , _____ , _____ , _____ , _____ ],
];

#[rustfmt::skip]
/// Mouse layer
const L2: LayerMap = [
    [ _____ , F1    , F2    , F3    , F4    , F5    , _____ , /**/ _____ , F6    , F7    , F8    , F9    , F10   , F11   ],
    [ _____ , _____ , INSERT, HOME  , PGUP  , _____ , _____ , /**/ _____ , LEFT  , DOWN  , UP    , RIGHT , _____ , F12   ],
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____,AML_RESET,M_LEFT,MO_SCRL,M_RIGHT, _____ , _____ ],
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , _____ ,M_BACK,M_MIDDLE,M_FORWARD,_____, _____ ],
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ DELETE, _____ , _____ , _____ , _____ , PRTSC , _____ ],
];

#[rustfmt::skip]
const L3: LayerMap = [
    [ FL_CLR, _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , _____ , _____ , _____ , _____ , _____ , _____ ],
    [ _____ , _____ , KP7   , KP8   , KP9   , _____ , _____ , /**/ _____ , SF(D1), SF(D2), SF(D3), SF(D4), SF(D5), _____ ],
    [ _____ , _____ , KP4   , KP5   , KP6   , _____ , _____ , /**/ _____ , SF(D6), SF(D7), SF(D8), SF(D9), SF(D0), _____ ],
    [ _____ , _____ , KP1   , KP2   , KP3   , _____ , _____ , /**/ _____ , QUOTE,SF(QUOTE),EQUAL,SF(EQUAL), _____ , _____ ],
    [ _____ , _____ , KP0   , _____ , _____ , _____ , _____ , /**/ _____ , _____ , _____ , _____ , _____ , _____ , _____ ],
];

#[rustfmt::skip]
const L4: LayerMap = [
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , _____ , _____ , _____ , _____ , _____ , _____ ],
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , _____ , _____ , _____ , _____ , _____ , _____ ],
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , _____ , _____ , _____ , _____ , _____ , _____ ],
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , _____ , _____ , _____ , _____ , _____ , _____ ],
    [ _____ , _____ , _____ , _____ , _____ , _____ , _____ , /**/ _____ , _____ , _____ , _____ , _____ , _____ , _____ ],
];

pub const KEYMAP: Keymap = Keymap {
    encoder_keys: [],
    layers: [
        Layer {
            map: L0,
            arrowmouse: false,
        },
        Layer {
            map: L1,
            arrowmouse: false,
        },
        Layer {
            map: L2,
            arrowmouse: false,
        },
        Layer {
            map: L3,
            arrowmouse: true,
        },
        Layer {
            map: L4,
            arrowmouse: true,
        },
    ],
    tap_dance: [
        Some(TapDanceDefinition {
            tap: [
                Some(KeyCode::Key(Key::RightBracket)),
                Some(KeyCode::Layer(LayerOp::Toggle(2))),
                None,
                None,
            ],
            hold: [None, None, None, None],
        }),
        None,
        None,
        None,
    ],
    combo: [None, None, None, None],
};
