use tracing_subscriber::EnvFilter;

/// 初始化日志系统。
///
/// Our own code uses `tracing` macros directly (`tracing::warn!`, `tracing::error!`, etc.).
/// Third-party dependencies (reqwest, hyper, etc.) still use the `log` crate.
/// The `tracing-log` `LogTracer` bridges them into the `tracing-subscriber` fmt layer.
///
/// Log level is controlled by the `RUST_LOG` environment variable, default `info`.
pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    if tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .try_init()
        .is_err()
    {
        tracing::error!("Failed to set global tracing subscriber");
    }

    if let Err(e) = tracing_log::LogTracer::init() {
        tracing::error!("Failed to initialize LogTracer: {}", e);
    }
}
