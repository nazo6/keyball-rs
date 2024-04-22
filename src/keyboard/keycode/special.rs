use super::{macros::with_consts_no_val, KeyCode};

with_consts_no_val!(
    Special,
    #[derive(PartialEq, Eq, Clone, Copy)]
    pub enum Special {
        MoScrl,
    }
);
