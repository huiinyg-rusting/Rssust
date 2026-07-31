# About Config
# 关于配置

The server reads `config.toml` from the same directory as the executable.
If the file does not exist, it will be auto-created with default content.

服务器从可执行文件同目录读取 `config.toml`。如果文件不存在，会在首次启动时自动创建默认配置。

## Format / 格式

```toml
[server]
port = 7878

[routes]
disabled = []
numofcore = 4
```

## Fields / 字段说明

| Field / 字段 | Type / 类型 | Default / 默认值 | Description / 描述 |
|--------------|-------------|------------------|---------------------|
| `server.port` | `int` | `7878` | 服务器监听端口。Port the server listens on. |
| `routes.disabled` | `string[]` | `[]` | 需要禁用的路由名列表。List of route names to disable. |
| `routes.numofcore` | `int` | 自动检测的 CPU 核心数 | 线程池工作线程数。Number of threads used by the thread pool. |

## Examples / 示例

禁用 `zhihu_hot` 和 `bilibili_weekly` 两个路由，并手动指定 8 个线程：

```toml
[server]
port = 8080

[routes]
disabled = ["zhihu_hot", "bilibili_weekly"]
numofcore = 8
```

不指定 `numofcore` 时，程序自动使用检测到的 CPU 核心数；不指定 `port` 时默认 `7878`：

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
