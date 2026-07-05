use Rssust::connect::handle_connection;
use std::net::TcpListener;
use threadpool::ThreadPool;

///main函数
/// 加载服务器
/// 启动threadpool多线程
fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(move || {
            handle_connection(stream);
        });
    }
}
