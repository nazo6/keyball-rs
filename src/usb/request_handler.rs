use defmt_rtt as _;
use embassy_usb::class::hid::{ReportId, RequestHandler};
use embassy_usb::control::OutResponse;

// use crate::utils::print_sync;

pub struct UsbRequestHandler {}

impl RequestHandler for UsbRequestHandler {
    fn get_report(&self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        // print_sync!("Get rep {:?}", id);
        None
    }

    fn set_report(&self, id: ReportId, data: &[u8]) -> OutResponse {
        // print_sync!("Set rep {:?}: {:?}", id, data);
        OutResponse::Accepted
    }

    fn set_idle_ms(&self, id: Option<ReportId>, dur: u32) {
        // print_sync!("S idle rate {:?}", dur);
    }

    fn get_idle_ms(&self, id: Option<ReportId>) -> Option<u32> {
        // print_sync!("G idle rate {:?}", id);
        None
    }
}
