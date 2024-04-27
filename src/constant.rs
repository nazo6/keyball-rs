pub const RIGHT_LED_NUM: usize = 34;
pub const LEFT_LED_NUM: usize = 37;

pub const SPLIT_BITRATE: f64 = 100000.0;
pub const SPLIT_CLK_DIVIDER: f64 = 125_000_000.0 / (SPLIT_BITRATE * 8.0);
pub const SPLIT_CHANNEL_SIZE: usize = 64;
/// Time (msec) to wait for USB connection to determine master/slave
pub const SPLIT_USB_TIMEOUT: u64 = 200;

pub const SCAN_COLS: usize = 4;
pub const SCAN_ROWS: usize = 5;

/// Time (msec) to wait for the next scan
pub const MIN_KB_SCAN_INTERVAL: u64 = 10;
pub const MIN_MOUSE_SCAN_INTERVAL: u64 = 5;

// Number of columns for one hand
pub const COLS: usize = 7;
// Number of rows for one hand
pub const ROWS: usize = 5;

pub const LEFT_DETECT_JUMPER_KEY: (usize, usize) = (2, 6);

pub const LAYER_NUM: usize = 4;

pub const AUTO_MOUSE_TIME: u64 = 500;
pub const AUTO_MOUSE_LAYER: usize = 1;

pub const SCROLL_DIVIDER: i8 = -8;

pub const TAP_THRESHOLD: u64 = 200;
