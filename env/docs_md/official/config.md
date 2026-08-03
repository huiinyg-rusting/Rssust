# About Config
# 关于配置

The server reads `config.toml` from the same directory as the executable.
If the file does not exist, it will be auto-created with default content.

服务器从可执行文件同目录读取 `config.toml`。如果文件不存在，会在首次启动时自动创建默认配置。

## Format / 格式

```toml
[server]
port = 7878
max_concurrent = 8
timeout = 60

[routes]
disabled = []
rate_limit = { "/scientificamerican_news" = 5, "/defenseone_news" = 1 }
```

## Fields / 字段说明

| Field / 字段 | Type / 类型 | Default / 默认值 | Description / 描述 |
|--------------|-------------|------------------|---------------------|
| `server.port` | `int` | `7878` | 服务器监听端口。Port the server listens on. |
| `server.max_concurrent` | `int` | 自动检测的 CPU 核心数 | 并发上限基数，实际许可数为该值 × 2。Base value of the concurrency limit; the actual number of permits is this value × 2. |
| `server.timeout` | `int` | `60` | 上游请求超时（秒）。Timeout for upstream requests, in seconds. |
| `routes.disabled` | `string[]` | `[]` | 需要禁用的路由名列表。List of route names to disable. |
| `routes.rate_limit` | `table` | `{}` | 路由名到缓存间隔（秒）的映射。配置后该路由的上游响应会被缓存；间隔内的重复请求直接复用缓存、不再请求上游；若解析报错则自动清理缓存。Map of route name to cache interval (seconds); caches upstream responses and avoids re-fetching within the interval, auto-cleared on parse errors. |

## Examples / 示例

禁用 `zhihu_hot` 和 `bilibili_weekly` 两个路由，并手动指定并发上限与超时：

```toml
[server]
port = 8080
max_concurrent = 4
timeout = 30

[routes]
disabled = ["zhihu_hot", "bilibili_weekly"]
```

不指定 `max_concurrent` 时，程序自动使用检测到的 CPU 核心数；不指定 `timeout` 时默认 60 秒；不指定 `port` 时默认 `7878`：

```toml
[routes]
disabled = []
```

## Logging / 日志

The program uses `tracing` as the logging backend. Our own code uses `tracing` macros directly
(`tracing::warn!`, `tracing::error!`, etc.). The `tracing-log` bridge forwards `log` crate records
from third-party dependencies (reqwest, hyper, etc.) into the same `tracing` subscriber.

程序使用 `tracing` 作为日志后端。项目自身代码直接使用 `tracing` 宏
（`tracing::warn!`、`tracing::error!` 等）；第三方依赖（reqwest、hyper 等）仍使用
`log` crate，通过 `tracing-log` 桥接统一转发到同一个 `tracing` 订阅器输出。

Log level is controlled by the `RUST_LOG` environment variable, default is `info`.
You can also set a per-target level, e.g. `RUST_LOG=info,rssust=debug`.

日志级别通过环境变量 `RUST_LOG` 控制，默认级别为 `info`。
也可以单独指定某个模块的级别，如 `RUST_LOG=info,rssust=debug`。

```bash
# 默认 info 级别
./rssust

# 显示 debug 级别日志
RUST_LOG=debug ./rssust

# 只显示 warn 及以上的日志
RUST_LOG=warn ./rssust

# 本项目模块用 debug，第三方库保持 info
RUST_LOG=info,rssust=debug ./rssust
```
