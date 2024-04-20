use super::{macros::with_consts_no_val, Keycode};

with_consts_no_val!(
    Special,
    #[derive(PartialEq, Eq, Clone, Copy)]
    pub enum Special {
        Inherit,
        None,
        MoScrl,
    }
);

pub const _____: Keycode = Keycode::Special(Special::Inherit);
