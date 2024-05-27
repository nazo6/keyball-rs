use embassy_time::Duration;

/* 内部設定。通常は変えないことをお勧めします。 */
pub const COL_PIN_NUM: usize = 4;
pub const SCAN_PIN_NUM: usize = 5;

pub const SPLIT_BITRATE: f64 = 100000.0;
pub const SPLIT_CLK_DIVIDER: f64 = 125_000_000.0 / (SPLIT_BITRATE * 8.0);
pub const SPLIT_CHANNEL_SIZE: usize = 64;

pub const LEFT_DETECT_JUMPER_KEY: (usize, usize) = (2, 6);

/* キーボード固有設定。keyball61以外のボードではこれを変える必要があります */

// Number of columns for one hand
pub const COLS: usize = 7;
// Number of rows for one hand
pub const ROWS: usize = 5;

pub const RIGHT_LED_NUM: usize = 34;
pub const LEFT_LED_NUM: usize = 37;

/* 変更するとパフォーマンスなどに影響を与える可能性がある設定 */
pub const USB_POLL_INTERVAL_KEYBOARD: u8 = 5;
pub const USB_POLL_INTERVAL_MOUSE: u8 = 5;

/// Time (msec) to wait for the next scan
pub const MIN_KB_SCAN_INTERVAL: Duration = Duration::from_millis(5);
pub const MIN_MOUSE_SCAN_INTERVAL: Duration = Duration::from_millis(5);

/* お好みで変えると良い設定 */

/// Time to wait for USB connection to determine master/slave
pub const SPLIT_USB_TIMEOUT: Duration = Duration::from_millis(200);

/// Layer count
pub const LAYER_NUM: usize = 4;

pub const AUTO_MOUSE_DURATION: Duration = Duration::from_millis(500);
pub const AUTO_MOUSE_LAYER: usize = 1;

pub const SCROLL_DIVIDER_X: i8 = 20;
pub const SCROLL_DIVIDER_Y: i8 = -12;

pub const TAP_THRESHOLD: Duration = Duration::from_millis(200);
