pub const SLAVE_LEDS_NUM: usize = 1;
pub const MASTER_LEDS_NUM: usize = 1;

pub const SPLIT_BITRATE: f64 = 100000.0;
pub const SPLIT_CLK_DIVIDER: f64 = 125_000_000.0 / (SPLIT_BITRATE * 8.0);
pub const SPLIT_CHANNEL_SIZE: usize = 10;
/// Time (msec) to wait for USB connection to determine master/slave
pub const SPLIT_USB_TIMEOUT: u64 = 300;

pub const SCAN_COLS: usize = 4;
pub const SCAN_ROWS: usize = 5;

/// Time (msec) to wait for the next scan
pub const MIN_SCAN_INTERVAL: u64 = 10;

// Number of columns for one hand
pub const COLS: usize = 7;
// Number of rows for one hand
pub const ROWS: usize = 5;

pub const LEFT_DETECT_JUMPER_KEY: (usize, usize) = (2, 6);

pub const LAYER_NUM: usize = 2;
