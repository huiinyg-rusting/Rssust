use bench_scraper::KnownBrowser;
use rssust::{
    // crawler::load_cookies,
    config, connect::handle_connection, cookies::extract_cookies_to_json,
    doc::doc_generate,
};
use std::env;
use std::net::TcpListener;
use threadpool::ThreadPool;
use log::{warn,trace};

///main函数
/// 加载服务器
/// 启动threadpool多线程
fn main() {
    config::init();
    let args: Vec<String> = env::args().collect();
    if matches!(args.get(1), Some(s) if s == "docs") {
        doc_generate().unwrap();
        println!("DOCS:Done")
    } else if matches!(args.get(1), Some(s) if s == "cookie") {
        extract_cookies_to_json(match args.get(2).expect("没有指明浏览器").as_str() {
            "firefox" => KnownBrowser::Firefox,
            "chrome" => KnownBrowser::Chrome,
            "chromium" => KnownBrowser::Chromium,
            "chromebeta" => KnownBrowser::ChromeBeta,
            #[cfg(target_os = "macos")]
            "safari" => KnownBrowser::Safari,
            #[cfg(target_os = "windows")]
            "edge" => KnownBrowser::Edge,
            _ => panic!("浏览器未知"),
        })
        .unwrap();
    }
    warn!("No args Find");
    // load_cookies().expect("cookies加载失败");
    trace!("启动端口监听和Obscura");
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(move || {
            handle_connection(stream);
        });
    }
}
