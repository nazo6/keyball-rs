#![allow(unused)]

macro_rules! print {
    ($lit:literal) => {{
        $crate::display::DISPLAY.set_message($lit).await;
    }};
    ($($arg:tt)*) => {{
        use core::fmt::Write;

        let mut str = heapless::String::<256>::new();
        write!(str, $($arg)*).unwrap();
        $crate::display::DISPLAY.set_message(&str).await;
    }}
}

macro_rules! print_sync {
    ($lit:literal) => {{
        $crate::display::DISPLAY.try_set_message($lit);
    }};
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut str = heapless::String::<256>::new();
        write!(str, $($arg)*).unwrap();
        $crate::display::DISPLAY.try_set_message(&str);
    }}
}

pub(crate) use print;
pub(crate) use print_sync;
