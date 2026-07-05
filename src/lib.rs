///这个函数提供缓冲区的处理
/// 并把数据交给request_rules函数处理
/// 最终在这个函数体内发送http数据
pub mod connect {
    use anyhow::Error;
    use anyhow::{Result, anyhow};
    use std::fs;
    use std::io::prelude::*;
    use std::net::TcpStream;
    use std::path::Path;

    pub fn handle_connection(mut stream: TcpStream) {
        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();

        let response = request_rules(buffer); //.expect("在加载request_rules时发生错误");
        let response = match response {
            Ok(i) => format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                i.len(),
                i
            ),
            Err(i) => format!("HTTP/1.1 200 OK\r\n\r\nError:{}", i),
        };

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }

    ///这个函数相当于模块的注册表
    /// 给调用者的是html格式
    fn request_rules(buffer: [u8; 1024]) -> Result<String, Error> {
        if buffer.starts_with(b"GET / HTTP/1.1\r\n") {
            show_index_doc()
        } else if buffer.starts_with(b"GET /what HTTP/1.1\r\n") {
            Ok("What is this?".to_string())
        } else {
            Err(anyhow!("404NotFound"))
        }
    }

    ///这个函数发送index.html的内容给调用者，否则发送错误及anyhow的文本错误类型 给调用者
    /// 给调用者的是html格式
    fn show_index_doc() -> Result<String, Error> {
        match fs::read_to_string(&Path::new("index.html")) {
            Ok(i) => Ok(i),

            Err(i) => Err(anyhow!(format!("{}:{}", "index.html", i.kind()))),
        }
    }
}