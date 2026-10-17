use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex, OnceLock};
use std::time::Instant;

static START: OnceLock<Instant> = OnceLock::new();
static REQUESTS: AtomicU64 = AtomicU64::new(0);
static FAILURES: AtomicU64 = AtomicU64::new(0);
static EVERYREQUESTS: LazyLock<Mutex<HashMap<String, u64>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn start_instant() -> Instant {
    *START.get_or_init(Instant::now)
}

pub fn record_request(key: String) {
    let mut requests = EVERYREQUESTS.lock().unwrap();
    requests
        .entry(key)
        .and_modify(|count| *count += 1)
        .or_insert(1);
}

pub fn every_requests() -> serde_json::Value {
    let requests = EVERYREQUESTS.lock().unwrap();
    serde_json::to_value(&*requests)
        .unwrap_or_else(|_| serde_json::Value::Object(Default::default()))
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
