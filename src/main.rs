use rssust::{connect::handle_connection, crawler::load_cookies};
use std::net::TcpListener;
use threadpool::ThreadPool;

///main函数
/// 加载服务器
/// 启动threadpool多线程
fn main() {
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
