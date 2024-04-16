/// Read the keyboard matrix and return a KeyboardReport.
/// If no keys are pressed, returns None.
pub async fn read(&mut self, other_side_keys: &[Option<(u8, u8)>; 6]) -> Option<KeyboardReport> {
    let keys = self.read_matrix().await;

    let mut keycodes = [0; 6];
    let mut idx = 0;

    for key in other_side_keys.iter() {
        if idx >= keycodes.len() {
            break;
        }
        if let Some((row, col)) = key {
            if row >= &5 || col >= &4 {
                continue;
            }
            let kc = keymap::KEYMAP[*row as usize][*col as usize];
            if kc == KC_NO {
                continue;
            }
            keycodes[idx] = kc;
            idx += 1;
        }
    }

    for (row, col) in keys.iter() {
        if idx >= keycodes.len() {
            break;
        }
        let kc = keymap::KEYMAP[*row as usize][*col as usize + 7];
        if kc == KC_NO {
            continue;
        }
        keycodes[idx] = kc;
        idx += 1;
    }

    if idx == 0 {
        return None;
    }

    Some(KeyboardReport {
        keycodes,
        leds: 0,
        modifier: 0,
        reserved: 0,
    })
}
