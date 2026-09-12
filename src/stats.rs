use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

static START: OnceLock<Instant> = OnceLock::new();
static REQUESTS: AtomicU64 = AtomicU64::new(0);
static FAILURES: AtomicU64 = AtomicU64::new(0);

fn start_instant() -> Instant {
    *START.get_or_init(Instant::now)
}

pub fn inc_request() {
    REQUESTS.fetch_add(1, Ordering::Relaxed);
}

pub fn inc_failure() {
    FAILURES.fetch_add(1, Ordering::Relaxed);
}

pub fn requests() -> u64 {
    REQUESTS.load(Ordering::Relaxed)
}

pub fn failures() -> u64 {
    FAILURES.load(Ordering::Relaxed)
}

pub fn up_secs() -> u64 {
    start_instant().elapsed().as_secs()
}
