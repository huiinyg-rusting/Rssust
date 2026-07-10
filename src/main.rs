use rssust::{connect::handle_connection, crawler::load_cookies, doc::doc_generate};
use std::env;
use std::net::TcpListener;
use threadpool::ThreadPool;

///main函数
/// 加载服务器
/// 启动threadpool多线程
fn main() {
    let args: Vec<String> = env::args().collect();
    if matches!(args.get(1), Some(s) if s == "docs") {
        doc_generate().unwrap();
        println!("DOCS:Done")
    };
    load_cookies().expect("cookies加载失败");
    println!("now,test the obscura");
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(move || {
            handle_connection(stream);
        });
    }
}
