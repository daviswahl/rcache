use std::sync::{Arc, atomic};
/// `Stats` middleware
///
#[derive(Default)]
pub struct Stats {
    total_requests: Arc<atomic::AtomicUsize>,
    total_request_time: Arc<atomic::AtomicUsize>,
}

impl Stats {
    pub fn incr_total_requests(&self) {
        self.total_requests.fetch_add(1, atomic::Ordering::SeqCst);
    }

    pub fn add_request_time(&self, millis: usize) {
        self.total_request_time.fetch_add(
            millis,
            atomic::Ordering::SeqCst,
        );
    }

    pub fn get_stats(&self) -> String {
        let total_requests = self.total_requests.load(atomic::Ordering::SeqCst);
        let total_requests_time = self.total_request_time.load(atomic::Ordering::SeqCst);
        let avg_request_time = total_requests_time / total_requests;
        format!(
            "total_requests: {}, total_request_time: {} millis, avg_request_time: {} millis",
            total_requests,
            total_requests_time,
            avg_request_time
        )
    }
}
