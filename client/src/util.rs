#[macro_export]
macro_rules! console_info {
    ($($t:tt)*) => (web_sys::console::info_1(&format!($($t)*).into()))
}

#[macro_export]
macro_rules! console_warn {
    ($($t:tt)*) => (web_sys::console::warn_1(&format!($($t)*).into()))
}

#[macro_export]
macro_rules! console_error {
    ($($t:tt)*) => (web_sys::console::error_1(&format!($($t)*).into()))
}
