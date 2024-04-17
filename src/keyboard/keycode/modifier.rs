#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Modifier {
    LeftCtrl = 0x00E0,
    LeftShift = 0x00E1,
    LeftAlt = 0x00E2,
    LeftGui = 0x00E3,
    RightCtrl = 0x00E4,
    RightShift = 0x00E5,
    RightAlt = 0x00E6,
    RightGui = 0x00E7,
}
