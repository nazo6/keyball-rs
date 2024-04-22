use embassy_futures::join::join;
use embassy_usb::{class::hid::HidWriter, driver::Driver};
use usbd_hid::descriptor::{KeyboardReport, MouseReport};

pub struct StateReport {
    pub keyboard_report: Option<KeyboardReport>,
    pub mouse_report: Option<MouseReport>,
}

impl StateReport {
    pub async fn report<'a, D: Driver<'a>, const N: usize>(
        &self,
        keyboard_writer: &mut HidWriter<'a, D, N>,
        mouse_writer: &mut HidWriter<'a, D, N>,
    ) {
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
        );
    }
}
