pub const SLAVE_LEDS_NUM: usize = 1;
pub const MASTER_LEDS_NUM: usize = 1;

pub const SPLIT_BITRATE: f64 = 115200.0;
pub const SPLIT_CLK_DIVIDER: f64 = 125_000_000.0 / (SPLIT_BITRATE * 8.0);
pub const SPLIT_CHANNEL_SIZE: usize = 10;

/// 片手キーボードの行数
pub const COLS: usize = 7;
/// 片手キーボードの列数
pub const ROWS: usize = 5;
