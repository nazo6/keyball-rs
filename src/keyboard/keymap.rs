use self::layer::MOVE;

use super::keycode::key::*;
use super::keycode::modifier::*;
use super::keycode::mouse::*;
use super::keycode::*;
use super::state::Layer;

#[rustfmt::skip]
pub const KEYMAP: [Layer; 2] = [[
    [____  , D1   , D2   , D3    , D4   , D5   , ____ , ____ , D6   , D7   , D8   , D9   ,D0    ,JBSLSH2],
    [TAB   , Q    , W    , E     , R    , T    , ____ , ____ , Y    , U    , I    , O    ,P     ,MINUS  ],
    [ESC   , A    , S    , D     , F    , G    , ____ , ____ , H    , J    , K    , L    ,SEMI  ,JCOLN  ],
    [L_SHFT, Z    , X    , C     , V    , B    , ____ , ____ , N    , M    , COMM , DOT  ,SLASH ,JBSLSH ],
    [L_CTRL, L_GUI, ____ ,MOVE(1), L_ALT, SPACE, SPACE, BS   , ENTER, ____ , ____ , ____ ,JCARET,JAT    ],
],[
    [ ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ ],
    [ ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ ],
    [ ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ , M_L  , M_MID, M_R  , ____ , ____ ],
    [ ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ ],
    [ ____ , ____ , ____ ,MOVE(0), ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ , ____ ],
]];
