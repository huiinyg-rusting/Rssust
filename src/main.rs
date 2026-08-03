use std::env;
use tokio::net::TcpListener;
use tracing::{error, info, warn};

use rssust::{config, connect::handle_connection};

#[cfg(feature = "cookie")]
use bench_scraper::KnownBrowser;
#[cfg(feature = "cookie")]
use rssust::cookies::extract_cookies_to_json;
#[cfg(feature = "docs")]
use rssust::doc::doc_generate;

///main函数
/// 加载服务器
/// 启动tokio多线程
#[tokio::main]
async fn main() {
    rssust::logger::init();
    config::init();
    let args: Vec<String> = env::args().collect();
    let subcommand = args.get(1).map(String::as_str);

    #[cfg(feature = "docs")]
    if subcommand == Some("docs") {
        info!("Generating docs HTML");
        match doc_generate() {
            Ok(()) => info!("DOCS:Done"),
            Err(e) => error!("Docs generation failed: {}", e),
        }
        return;
    }
    #[cfg(feature = "cookie")]
    if matches!(subcommand, Some("cookie" | "cookies")) {
        info!("Exporting browser cookies");
        match extract_cookies_to_json(match args.get(2).expect("没有指明浏览器").as_str() {
            "firefox" => KnownBrowser::Firefox,
            "chrome" => KnownBrowser::Chrome,
            "chromium" => KnownBrowser::Chromium,
            "chromebeta" => KnownBrowser::ChromeBeta,
            #[cfg(target_os = "macos")]
            "safari" => KnownBrowser::Safari,
            #[cfg(target_os = "windows")]
            "edge" => KnownBrowser::Edge,
            _ => panic!("浏览器未知"),
        }) {
            Ok(()) => info!("Cookies exported successfully"),
            Err(e) => error!("Cookie export failed: {}", e),
        }
        return;
    }
    if let Some(cmd) = subcommand {
        eprintln!("Unknown subcommand: {}", cmd);
        print_usage();
        std::process::exit(1);
    }
    let port = config::server_port();
    let addr = format!("127.0.0.1:{}", port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind port {}: {}", port, e);
            std::process::exit(1);
        }
    };
    info!("Starting server, listening on {}", addr);
    rssust::rate_limit::spawn_cleaner();
    loop {
        let (stream, _) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to accept connection: {}", e);
                continue;
            }
        };
        tokio::spawn(handle_connection(stream));
    }
}

fn print_usage() {
    eprintln!("Usage: rssust [docs] [cookie <browser>]");
    eprintln!("  (no args)           start the RSS server");
    #[cfg(feature = "docs")]
    eprintln!("  docs                generate documentation HTML");
    #[cfg(feature = "cookie")]
    eprintln!("  cookie <browser>    export browser cookies (firefox/chrome/chromium/chromebeta)");
}
