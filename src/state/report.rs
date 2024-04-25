use embassy_futures::join::join;
use embassy_usb::{class::hid::HidWriter, driver::Driver};
use usbd_hid::descriptor::{KeyboardReport, MouseReport};

pub struct StateReport {
    pub keyboard_report: Option<KeyboardReport>,
    pub mouse_report: Option<MouseReport>,
    pub highest_layer: u8,
}

impl StateReport {
    /// Returns true if any report is sent.
    pub async fn report<'a, D: Driver<'a>, const N: usize>(
        &self,
        keyboard_writer: &mut HidWriter<'a, D, N>,
        mouse_writer: &mut HidWriter<'a, D, N>,
    ) -> bool {
        join(
            async {
                if let Some(report) = &self.keyboard_report {
                    let _ = keyboard_writer.write_serialize(report).await;
                }
            },
            async {
                if let Some(report) = &self.mouse_report {
                    let _ = mouse_writer.write_serialize(report).await;
                }
            },
        )
        .await;

        self.keyboard_report.is_some() || self.mouse_report.is_some()
    }
}
