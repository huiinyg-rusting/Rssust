pub mod config;
#[cfg(feature = "cookie")]
pub mod cookies;
#[cfg(feature = "docs")]
pub mod doc;
pub mod easyuser;
pub mod logger;
pub mod request_rules;
pub mod router;

///这个函数提供缓冲区的处理
/// 并把数据交给request_rules函数处理
/// 最终在这个函数体内发送http数据
pub mod connect {
    use crate::easyuser::HttpError;
    use crate::request_rules::*;
    use anyhow::{Context, Error, anyhow};
    use std::collections::HashMap;
    use std::env;
    use std::path::Path;
    use std::sync::OnceLock;
    use std::time::Duration;
    use tokio::fs;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio::sync::Semaphore;
    use tracing::{debug, info, warn};

    const MAX_HEAD_SIZE: usize = 16 * 1024;
    const IDLE_TIMEOUT: Duration = Duration::from_secs(30);

    fn semaphore() -> &'static Semaphore {
        static SEM: OnceLock<Semaphore> = OnceLock::new();
        SEM.get_or_init(|| {
            let permits = crate::config::max_concurrent() as usize;
            info!("Concurrency limit: {} permits", permits);
            Semaphore::new(permits)
        })
    }

    ///每个 TCP 连接的入口。读取请求、调度路由、写回响应，并在 keep-alive 下循环复用连接。
    pub async fn handle_connection(mut stream: TcpStream) {
        let mut buf: Vec<u8> = Vec::with_capacity(1024);
        loop {
            let head_len = match read_head(&mut stream, &mut buf).await {
                Ok(Some(len)) => len,
                Ok(None) => return,
                Err(e) => {
                    match e.kind() {
                        std::io::ErrorKind::TimedOut => {
                            debug!("Idle connection timed out while reading request head");
                        }
                        std::io::ErrorKind::UnexpectedEof => {
                            debug!("Client closed connection before request head completed");
                        }
                        _ => warn!("Failed to read request head: {}", e),
                    }
                    return;
                }
            };
            let head_bytes = buf[..head_len].to_vec();
            let head_str = match std::str::from_utf8(&head_bytes) {
                Ok(s) => s,
                Err(_) => {
                    warn!("Request head is not valid UTF-8");
                    if let Err(e) = write_response(
                        &mut stream,
                        400,
                        "text/plain; charset=utf-8",
                        b"Bad Request",
                        false,
                    )
                    .await
                    {
                        warn!("Failed to write response: {}", e);
                    }
                    return;
                }
            };
            let (method, target, version) = match parse_request_line(head_str) {
                Some(t) => t,
                None => {
                    warn!("Malformed request line");
                    if let Err(e) = write_response(
                        &mut stream,
                        400,
                        "text/plain; charset=utf-8",
                        b"Bad Request",
                        false,
                    )
                    .await
                    {
                        warn!("Failed to write response: {}", e);
                    }
                    return;
                }
            };
            if method != "GET" && method != "HEAD" {
                if let Err(e) = write_response(
                    &mut stream,
                    405,
                    "text/plain; charset=utf-8",
                    b"Method Not Allowed",
                    false,
                )
                .await
                {
                    warn!("Failed to write response: {}", e);
                }
                return;
            }
            let version = version.to_string();

            let (path, params) = if let Some((before, after)) = target.split_once('?') {
                (
                    before.to_string(),
                    crate::easyuser::params_to_hashmap(after),
                )
            } else {
                (target.to_string(), HashMap::new())
            };
            info!("Received request: {} params: {:?}", path, params);

            //保持连接（keep-alive）仅对文档/静态资源生效，其它动态路由一律关闭
            let is_doc_route = path == "/"
                || path.starts_with("/docs/")
                || path.starts_with("/index/")
                || path == "/favicon.ico";
            let keep_alive = is_doc_route && parse_keep_alive(head_str, &version);

            //并发上限：请求处理前获取许可
            let _permit = match semaphore().acquire().await {
                Ok(p) => p,
                Err(_) => {
                    warn!("Semaphore closed");
                    return;
                }
            };

            //路由（async）+ 渲染在 tokio worker 线程执行，信号量限流并发数
            let path_owned = path.clone();
            let result = tokio::spawn(async move {
                let resp = root_rules(&path_owned, params).await;
                render(resp)
            })
            .await;
            let (status, content_type, body) = match result {
                Ok(r) => r,
                Err(e) => {
                    warn!("Task join failed: {}", e);
                    (
                        500,
                        "text/plain; charset=utf-8".to_string(),
                        format!("Internal Server Error: {}", e).into_bytes(),
                    )
                }
            };

            if let Err(e) =
                write_response(&mut stream, status, &content_type, &body, keep_alive).await
            {
                warn!("Failed to write response: {}", e);
                return;
            }

            //消耗已读请求头，剩余字节留给 keep-alive 的下一个请求
            buf.drain(..head_len);
            if !keep_alive {
                break;
            }
        }
    }

    ///解析请求行 "METHOD TARGET VERSION"
    fn parse_request_line(head: &str) -> Option<(&str, &str, &str)> {
        let mut parts = head.lines().next()?.split_whitespace();
        let method = parts.next()?;
        let target = parts.next()?;
        let version = parts.next().unwrap_or("HTTP/1.1");
        Some((method, target, version))
    }

    ///读取完整请求头（到 `\r\n\r\n`）。返回 `Ok(None)` 表示对端干净关闭，`Ok(Some(len))` 为完整头部长度。
    async fn read_head(
        stream: &mut TcpStream,
        buf: &mut Vec<u8>,
    ) -> std::io::Result<Option<usize>> {
        loop {
            if let Some(pos) = find_head_end(buf) {
                return Ok(Some(pos + 4));
            }
            if buf.len() >= MAX_HEAD_SIZE {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Request head too large",
                ));
            }
            let mut tmp = [0u8; 1024];
            let n = tokio::time::timeout(IDLE_TIMEOUT, stream.read(&mut tmp))
                .await
                .map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::TimedOut, "Request head read timeout")
                })??;
            if n == 0 {
                //对端关闭连接：若缓冲为空则为干净关闭，否则是残缺请求
                if buf.is_empty() {
                    return Ok(None);
                }
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Connection closed before request head completed",
                ));
            }
            buf.extend_from_slice(&tmp[..n]);
        }
    }

    fn find_head_end(buf: &[u8]) -> Option<usize> {
        buf.windows(4).position(|w| w == b"\r\n\r\n")
    }

    ///解析 Connection 头判断是否保持连接。
    ///HTTP/1.1 默认 keep-alive，除非显式 `Connection: close`；HTTP/1.0 需要显式 `Connection: keep-alive`。
    fn parse_keep_alive(head: &str, version: &str) -> bool {
        let mut close = false;
        let mut keep = false;
        for line in head.lines() {
            let (key, value) = match line.split_once(':') {
                Some((k, v)) => (k.trim().to_ascii_lowercase(), v.trim().to_ascii_lowercase()),
                None => continue,
            };
            if key == "connection" {
                close |= value == "close";
                keep |= value == "keep-alive";
            }
        }
        if version.starts_with("HTTP/1.1") {
            !close
        } else {
            keep
        }
    }

    ///渲染：将 `ShowToUser` 转为 (状态码, Content-Type, 响应体)。
    ///错误兜底：命中 `HttpError` 则用其自定义状态码与正文，否则默认 500。
    fn render(response: ShowToUser) -> (u16, String, Vec<u8>) {
        match response {
            ShowToUser::Html { res } => match res {
                std::result::Result::Ok(i) => {
                    debug!("Returning HTML, length {}", i.len());
                    (200, "text/html; charset=utf-8".to_string(), i.into_bytes())
                }
                Err(e) => error_render(e),
            },
            ShowToUser::Rss { res } => match res {
                std::result::Result::Ok(i) => {
                    debug!("Returning RSS, length {}", i.len());
                    (
                        200,
                        "application/xml; charset=utf-8".to_string(),
                        i.into_bytes(),
                    )
                }
                Err(e) => error_render(e),
            },
            ShowToUser::File { res, content_type } => match res {
                std::result::Result::Ok(i) => {
                    debug!("Returning file ({}), length {}", content_type, i.len());
                    (200, content_type, i)
                }
                Err(e) => error_render(e),
            },
        }
    }

    ///错误渲染兜底：anyhow 错误中若能挖出 `HttpError` 则用其状态码+正文；`404NotFound` 视为 404；否则默认 500。
    fn error_render(e: Error) -> (u16, String, Vec<u8>) {
        warn!("Error requesting: {}", e);
        if let Some(he) = e.downcast_ref::<HttpError>() {
            return (
                he.status,
                "text/plain; charset=utf-8".to_string(),
                he.message.clone().into_bytes(),
            );
        }
        if e.to_string() == "404NotFound" {
            return (
                404,
                "text/plain; charset=utf-8".to_string(),
                "404NotFound".as_bytes().to_vec(),
            );
        }
        (
            500,
            "text/plain; charset=utf-8".to_string(),
            e.to_string().into_bytes(),
        )
    }

    fn reason_phrase(status: u16) -> &'static str {
        match status {
            200 => "OK",
            400 => "Bad Request",
            404 => "Not Found",
            405 => "Method Not Allowed",
            431 => "Request Header Fields Too Large",
            500 => "Internal Server Error",
            502 => "Bad Gateway",
            _ => "Error",
        }
    }

    async fn write_response(
        stream: &mut TcpStream,
        status: u16,
        content_type: &str,
        body: &[u8],
        keep_alive: bool,
    ) -> std::io::Result<()> {
        let connection = if keep_alive { "keep-alive" } else { "close" };
        let head = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: {}\r\n\r\n",
            status,
            reason_phrase(status),
            content_type,
            body.len(),
            connection
        );
        stream.write_all(head.as_bytes()).await?;
        stream.write_all(body).await?;
        stream.flush().await
    }

    ///This function sends the content of index.html to the caller; otherwise, it sends an error with an anyhow text error type.‌
    /// The response returned to the caller is in HTML format.
    pub async fn show_index_doc() -> Result<String, Error> {
        let exe_path = env::current_exe()?;
        let exe_dir = exe_path
            .parent()
            .ok_or_else(|| anyhow!("Could not get executable directory"))?;

        match fs::read_to_string(&Path::new(&exe_dir.join("index/index.html"))).await {
            std::result::Result::Ok(i) => Ok(i),

            Err(i) => Err(anyhow!(format!("{}:{}", "index.html", i.kind()))),
        }
    }
    //传入的像是/doc/new.html
    pub async fn show_doc(path: &str) -> Result<String, Error> {
        let exe_path = env::current_exe()?;

        let exe_dir = exe_path
            .parent()
            .ok_or_else(|| anyhow!("Could not get executable directory"))?;

        let mut raw = exe_dir.join(path.trim_matches('/'));
        if raw.is_dir() {
            raw = raw.join("index.html");
        }
        match fs::read_to_string(&raw).await {
            std::result::Result::Ok(i) => Ok(i),

            Err(_) => Ok(fs::read_to_string(exe_dir.join("docs/404.html"))
                .await
                .context("404 html Operation failed")?),
        }
    }

    async fn read_file_bytes(path: &str) -> Result<Vec<u8>, Error> {
        let exe_path = env::current_exe()?;
        let exe_dir = exe_path
            .parent()
            .ok_or_else(|| anyhow!("Could not get executable directory"))?;

        fs::read(exe_dir.join(path.trim_matches('/')))
            .await
            .map_err(|e| anyhow!(format!("{}:{}", path, e.kind())))
    }

    pub fn mime_type(path: &str) -> &'static str {
        if path.ends_with(".css") {
            "text/css; charset=utf-8"
        } else if path.ends_with(".js") {
            "application/javascript; charset=utf-8"
        } else if path.ends_with(".svg") {
            "image/svg+xml"
        } else if path.ends_with(".png") {
            "image/png"
        } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
            "image/jpeg"
        } else if path.ends_with(".gif") {
            "image/gif"
        } else if path.ends_with(".ico") {
            "image/x-icon"
        } else if path.ends_with(".woff2") {
            "font/woff2"
        } else if path.ends_with(".woff") {
            "font/woff"
        } else if path.ends_with(".ttf") {
            "font/ttf"
        } else if path.ends_with(".json") {
            "application/json; charset=utf-8"
        } else if path.ends_with(".xml") {
            "application/xml; charset=utf-8"
        } else {
            "text/html; charset=utf-8"
        }
    }

    pub async fn serve_static(path: &str) -> crate::request_rules::ShowToUser {
        let mime = mime_type(path);

        if mime == "text/html; charset=utf-8" {
            return crate::request_rules::ShowToUser::Html {
                res: show_doc(path).await,
            };
        }

        let res = read_file_bytes(path).await;
        match res {
            std::result::Result::Ok(_) => crate::request_rules::ShowToUser::File {
                res,
                content_type: mime.to_string(),
            },
            std::result::Result::Err(e) => {
                warn!("Failed to read static file {}: {}", path, e);
                crate::request_rules::ShowToUser::Html {
                    res: Err(anyhow!(format!("404 html Operation failed: {}", e))),
                }
            }
        }
    }
}
