# rhai 脚本路由（scripts feature）

Dynamic RSS routes written in [rhai](https://rhai.rs/) scripts. Compile with `--features scripts`,
drop a `.rhai` file into the `scripts/` directory next to the executable, and the route goes live
without touching Rust code.

用 rhai 脚本写动态 RSS 路由。编译时加 `--features scripts`，把 `.rhai` 文件丢进 exe 同目录的 `scripts/`
文件夹，路由即刻生效，不用改 Rust 代码。

## 启用 / Enable

```bash
# 带脚本路由编译
cargo build --release --features scripts

# 或者跟其他 feature 一起
cargo build --release --all-features
```

`scripts` 不在 `default` 里（`default = ["docs", "cookie"]`），所以不显式开启就没有这个能力。

## 脚本放哪 / Where scripts live

```
env/
├── rssust          # 可执行文件
└── scripts/        # 脚本目录，平铺，不分子文件夹
    ├── bilibili_partion_ranking_script.rhai  → /bilibili_partion_ranking_script
    └── bilibili_search_hot_script.rhai       → /bilibili_search_hot_script
```

路由名 = 文件名去掉 `.rhai`，命名跟静态路由一个风格（全小写下划线）；跟内置路由重名的加 `_script` 后缀，
`bilibili_search_hot_script` 就是内置 `bilibili_search_hot` 的脚本版。
`scripts/` 下不递归子目录，子文件夹里的脚本会被忽略。

## 什么时候生效 / When it runs

静态路由（`src/request_rules.rs` 里的 `routes!` 宏）先匹配，**没命中才回退到脚本路由**：

- 静态路由存在 → 走静态路由，脚本不会被执行（脚本不要跟静态路由重名，否则等于失效）
- 静态路由不存在，且 `scripts/` 里有同名文件 → 走脚本
- 两边都没有 → 404

`config.toml` 的 `routes.disabled` 与 `routes.rate_limit` 对脚本路由同样生效，键名要带前导斜杠：

```toml
[routes]
disabled = ["/bilibili_search_hot_script"]
rate_limit = { "/bilibili_partion_ranking_script" = 60 }
```

## 脚本怎么写 / Script anatomy

脚本是纯数据处理：**不发网络请求**。Rust 侧按 `//@source` 抓取上游，把响应文本注入 `raw`，脚本解析后 `return` 一个数组。

顶部元数据注释：

| 注释 | 必填 | 作用 |
|------|------|------|
| `//@name 标题` | 否 | RSS 频道标题；不写则用 `脚本 /路由名` |
| `//@source URL` | **是** | 上游抓取地址，支持 `{参数名}` 占位；缺失时请求返回 500 |
| `//@item 选择器` | 否 | 预留字段，目前不生效 |

频道简介（`<description>`）取 `let doc` 里的首个非标题行；没有 `let doc` 时用 `rhai 脚本路由 /路由名`。

`//@source` 里的 `{参数名}` 会被 URL 查询参数替换，例如 `/bilibili_partion_ranking_script?rid=36` 把 `{rid}` 换成 `36`。
没传的参数会原样留在 URL 里（`rid={rid}`），上游多半返回错误结构，表现为脚本执行失败：

```
脚本 /bilibili_partion_ranking_script 执行失败: Indexer unavailable: () [string] (line 15, position 25)
```

脚本里还可以写一段 markdown 文档（RSSHub 风格）：

```rhai
let doc = `# 标题

第一行非标题内容会作为 RSS 频道简介。

- 补充说明写在这里
`;
```

`rssust docs` 会从 `scripts/` 里提取这段 markdown 渲染成 HTML 文档页（不往仓库里写 `.md` 文件），
同时出现在文档站侧栏和路由清单的 `Scripts (脚本路由)` 分组里。

## 注入的变量 / Injected scope

| 变量 | 类型 | 说明 |
|------|------|------|
| `raw` | 字符串 | `//@source` 抓回来的原始响应体 |
| `params` | map | URL 查询参数，如 `params["rid"]` |

## 返回值 / Return value

必须 `return` 一个数组，每个元素是对象，字段按名字取，缺哪个都不报错：

| 字段 | 必填 | 说明 |
|------|------|------|
| `title` | 是 | 条目标题 |
| `link` | 是 | 条目链接 |
| `description` | 否 | 条目描述（HTML 片段，会放进 CDATA） |
| `pub_date` | 否 | 发布时间字符串，原样写进 `<pubDate>` |
| `author` | 否 | 作者 |

`title` 和 `link` 同时为空的条目会被丢弃；`description` 不给会输出空的 `<description><![CDATA[]]></description>`。

## 完整示例 / Full example

`scripts/bilibili_partion_ranking_script.rhai`：

```rhai
//@name B站分区排行榜(脚本示例)
//@source https://api.bilibili.com/x/web-interface/ranking/v2?rid={rid}&type=all

let doc = `# B站分区排行榜(脚本示例)

内置 bilibili_partion_ranking 的脚本版，换用官方 ranking/v2 接口，{rid} 传分区。

- 路由: /bilibili_partion_ranking_script?rid=36
- rid 取值: 0 全部 / 3 音乐 / 4 游戏 / 36 知识 / 188 科技 / 160 生活
`;

let data = parse_json(raw);
let result = [];
for item in data["data"]["list"] {
    let up = item["owner"]["name"];
    result.push(#{
        "title": item["title"],
        "link": "https://www.bilibili.com/video/" + item["bvid"],
        "description": "UP: " + up + " · 播放: " + item["stat"]["view"].to_string(),
        "pub_date": "",
        "author": up
    });
}
return result;
```

上游返回：

```json
{ "code": 0, "data": { "list": [ { "title": "...", "bvid": "BV...", "owner": { "name": "..." }, "stat": { "view": 123 } } ] } }
```

对应产出：

```xml
<channel>
  <title>B站分区排行榜(脚本示例)</title>
  <link>https://api.bilibili.com/x/web-interface/ranking/v2?rid=36&amp;type=all</link>
  <description>内置 bilibili_partion_ranking 的脚本版，换用官方 ranking/v2 接口，{rid} 传分区。</description>
  <item>
    <title>前8天：穿越罗布泊无人区  第9天：主角突然换人</title>
    <link>https://www.bilibili.com/video/BV1soZJYUEKd</link>
    <description><![CDATA[UP: 央视新闻 · 播放: 5159432]]></description>
    <author>央视新闻</author>
  </item>
</channel>
```

## 热重载与报错 / Hot reload & errors

- 服务器每 2 秒重扫一次 `scripts/`，文件 mtime 变了就重新编译，改脚本不用重启
- 新增 / 删除 `.rhai` 文件同样 2 秒内生效（删掉后路由回到 404）
- 语法错误不影响启动，其他路由照常工作，只有访问该路由时返回 500，错误信息带行号：

```
脚本 /bilibili_search_hot_script 编译错误: Unexpected ';' (line 3, position 9)
```

- 运行期报错（类型不对、上游数据结构变了、脚本 panic）同样返回 500 并说明原因，不会拖垮 tokio 线程：

```
脚本 /my_route 执行失败: Data type incorrect: string (expecting i64) (line 4, position 31)
```

## rhai 语法坑 / Gotchas

- map 字面量必须写 `#{ "k": "v" }`，不能写 `{ "k": "v" }`
- 没有三元运算符 `a ? b : c`，用 `if ... { } else { }`
- 取不存在的键得到 `()`，判断要用 `if x != () { ... }`
- JSON 用嵌套下标访问：`data["data"]["list"][0]["title"]`；点号取字段不支持，`d.data.list` 会报 `Unknown property`
- `parse_json` 依赖 rhai 的 `serde` feature（已在本项目开启）
- 脚本里没有时间格式化能力，`pub_date` 要自己拼字符串，且不会被校验

## 相关文档

- 路由清单与各路由说明：[routes](../routes.md)
- 新增静态路由：[new_router_cn](new_router_cn.md)
- 配置文件字段：[config](config.md)