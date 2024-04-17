use super::macros::with_consts;

with_consts!(
    Modifier,
    #[derive(PartialEq, Eq, Clone, Copy)]
    pub enum Modifier {
        LCtrl = 0x01,
        LShft = 0x02,
        LAlt = 0x04,
        LGui = 0x08,
        RCtrl = 0x10,
        RShft = 0x20,
        RAlt = 0x40,
        RGui = 0x80,
    }
);
