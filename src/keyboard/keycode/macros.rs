macro_rules! with_consts {
    {
        $key_type:ident,
        $(#[$($attr:tt)*])*
        $vis:vis enum $name:ident {
            $($variant:ident = $val:literal,)*
        }
    } => {
        $(#[$($attr)*])*
        $vis enum $name { $($variant = $val,)* }

        paste::paste!{
            $(pub const [<$variant:snake:upper>] : super::KeyAction = super::KeyAction::Normal(super::KeyCode::$key_type($name::$variant));)*
        }
    }
}

macro_rules! with_consts_no_val {
    {
        $key_type:ident,
        $(#[$($attr:tt)*])*
        $vis:vis enum $name:ident {
            $($variant:ident,)*
        }
    } => {
        $(#[$($attr)*])*
        $vis enum $name { $($variant,)* }

        paste::paste!{
            $(pub const [<$variant:snake:upper>] : super::KeyAction = super::KeyAction::Normal(super::KeyCode::$key_type($name::$variant));)*
        }
    }
}

macro_rules! normal {
    ($name:ident, $type:ident, $variant:ident) => {
        pub const $name: super::KeyAction =
            super::KeyAction::Normal(super::KeyCode::$type($type::$variant));
    };
}

pub(super) use normal;
pub(super) use with_consts;
pub(super) use with_consts_no_val;
