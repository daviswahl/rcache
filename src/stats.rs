use std::sync::{Arc, atomic};
/// `Stats` middleware
///
pub struct Stats {
    handled: atomic::AtomicUsize,
}

impl Stats {
    pub fn new() -> Self {
        Self{handled: atomic::AtomicUsize::new(0)}
    }
    pub fn incr_handled(&self) {
        println!("incre!");
        self.handled.fetch_add(1, atomic::Ordering::SeqCst);
    }

    pub fn get_stats(&self) -> String {
        format!("handled: {}", self.handled.load(atomic::Ordering::SeqCst))
    }
}
