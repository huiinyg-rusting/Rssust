use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use threadpool::ThreadPool;

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

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let response = request_rules(buffer);

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn request_rules(buffer: [u8; 1024]) -> String {
    let response: String = if buffer.starts_with(b"GET / HTTP/1.1\r\n") {
        show_index_doc()
    } else if buffer.starts_with(b"GET /what HTTP/1.1\r\n") {
        "HTTP/1.1 200 OK\r\n\r\nWhat is this?".to_string()
    } else {
        "HTTP/1.1 404 NOT FOUND\r\n\r\n".to_string()
    };
    response
}

fn show_index_doc() -> String {
    match fs::read_to_string(&Path::new("index.html")) {
        Ok(i) => {
            format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
                i.len(),
                i
            )
        }

        Err(i) => {
            format!(
                "HTTP/1.1 500 Internal Server Error\r\nContent-Length: \r\n\r\n{}:{}",
                "index.html",
                i.kind()
            )
        }
    }
}
