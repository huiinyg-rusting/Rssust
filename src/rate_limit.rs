use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tracing::{debug, info};

type CacheEntry = (Instant, String);

static CACHE: OnceLock<Mutex<HashMap<String, CacheEntry>>> = OnceLock::new();

struct CacheScopeCtx {
    ttl: Option<Duration>,
    touched: Mutex<Vec<String>>,
}

tokio::task_local! {
    static CACHE_CTX: CacheScopeCtx;
}

fn cache() -> &'static Mutex<HashMap<String, CacheEntry>> {
    CACHE.get_or_init(|| {
        info!("Rate-limit cache initialized");
        Mutex::new(HashMap::new())
    })
}

///构造缓存 key：方法 + URL + 排序后的 headers 指纹，保证同一上游请求 key 确定性。
pub fn make_key(method: &str, url: &str, headers: &[(&str, &str)]) -> String {
    if headers.is_empty() {
        return format!("{}|{}", method, url);
    }
    let mut hs: Vec<(String, String)> = headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    hs.sort();
    let h = hs
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");
    format!("{}|{}|{}", method, url, h)
}

///当前请求是否处于限流缓存作用域，返回其 TTL。
pub fn current_ttl() -> Option<Duration> {
    CACHE_CTX.try_with(|c| c.ttl).ok().flatten()
}

fn record_touched(key: &str) {
    let _ = CACHE_CTX.try_with(|c| {
        if let Ok(mut v) = c.touched.lock() {
            v.push(key.to_string());
        }
    });
}

///命中且未过期则返回缓存内容并记录；过期条目惰性删除。
pub fn get_cached(key: &str, ttl: Duration) -> Option<String> {
    let mut map = cache().lock().unwrap();
    match map.get(key) {
        Some((created, body)) if created.elapsed() < ttl => {
            let body = body.clone();
            drop(map);
            record_touched(key);
            Some(body)
        }
        _ => {
            map.remove(key);
            None
        }
    }
}

///写入缓存并记录本次使用。
pub fn store(key: &str, body: &str) {
    cache()
        .lock()
        .unwrap()
        .insert(key.to_string(), (Instant::now(), body.to_string()));
    record_touched(key);
}

///路由处理报错时调用：清理本次请求用到的所有缓存条目。
pub fn cleanup_on_error() {
    let keys = match CACHE_CTX.try_with(|c| std::mem::take(&mut *c.touched.lock().unwrap())) {
        Ok(keys) => keys,
        Err(_) => return,
    };
    if keys.is_empty() {
        return;
    }
    let mut map = cache().lock().unwrap();
    for k in &keys {
        map.remove(k);
    }
    debug!(
        "Cleared {} cached upstream responses after route error",
        keys.len()
    );
}

///后台定时清扫过期条目，防止缓存无限增长。
pub fn spawn_cleaner() {
    tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            let now = Instant::now();
            let mut map = cache().lock().unwrap();
            let before = map.len();
            map.retain(|_, (created, _)| now.duration_since(*created) < Duration::from_secs(600));
            if map.len() != before {
                debug!(
                    "Rate-limit cache swept: {} -> {} entries",
                    before,
                    map.len()
                );
            }
        }
    });
}

///以给定 TTL 包裹异步执行，路由处理期间 fetch 层可读取限流上下文。
pub async fn with_cache_scope<T>(
    ttl: Option<Duration>,
    f: impl std::future::Future<Output = T>,
) -> T {
    CACHE_CTX
        .scope(
            CacheScopeCtx {
                ttl,
                touched: Mutex::new(Vec::new()),
            },
            f,
        )
        .await
}
