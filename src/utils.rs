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

pub(crate) use print;
