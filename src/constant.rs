pub const SLAVE_LEDS_NUM: usize = 1;
pub const MASTER_LEDS_NUM: usize = 1;

pub const SPLIT_BITRATE: f64 = 400000.0;
pub const SPLIT_CLK_DIVIDER: f64 = 125_000_000.0 / (SPLIT_BITRATE * 8.0);
pub const SPLIT_CHANNEL_SIZE: usize = 10;
/// Time (msec) to wait for USB connection to determine master/slave
pub const SPLIT_USB_TIMEOUT: u64 = 300;

// Number of columns for one hand
pub const COLS: usize = 7;
// Number of rows for one hand
pub const ROWS: usize = 5;
