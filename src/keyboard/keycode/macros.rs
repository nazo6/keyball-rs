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
            $(pub const [<$variant:snake:upper>] : super::Keycode = super::Keycode::$key_type($name::$variant);)*
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
            $(pub const [<$variant:snake:upper>] : super::Keycode = super::Keycode::$key_type($name::$variant);)*
        }
    }
}

pub(super) use with_consts;
pub(super) use with_consts_no_val;
