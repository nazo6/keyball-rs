use embassy_time::Instant;

pub struct AmlController {}

impl AmlController {
    pub fn new() -> Self {
        Self { start: None }
    }
    pub fn update(&mut self) {}
}
