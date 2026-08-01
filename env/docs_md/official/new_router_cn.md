### [English HERE](new_router_en.md)
# 路由制作指南
本文可以给人看，也可以给AI看
在开发前，你需要：
- 一个标准的cargo环境
- git 版本管理系统
- 编辑器
- **基础**rust的知识,最好是通过官方书籍，以及有一些std库里频率很高的方法也要学会，比如给可恢复错误做处理的几种方法之类的;
- RSS的知识 可以通过：[官方RSS 2.0.11 英文](https://www.rssboard.org/rss-specification) 或者 [博客园-小y 翻译的RSS 2.0.10 中文](https://www.cnblogs.com/tuyile006/p/3691024.html)进行学习
- RSS Crates 的知识，懂得如何构建一个RSS [官方英文文档](https://docs.rs/rss/latest/rss/)
  **尤其**是[ChannelBuilder](https://docs.rs/rss/latest/rss/struct.ChannelBuilder.html)和[ImageBuilder](https://docs.rs/rss/latest/rss/struct.ImageBuilder.html)
- 因为不同上游API,你需要学会不同的方法，比如说你的上游API是json格式，你其实实际大概率只需要学会serde_json Crates的.pointer()和.get()这两种方法，而如果你需要直接处理html和DOM,你需要[scraper](https://docs.rs/scraper/0.27.0/scraper/)的文档中推荐的方法，以及避免导致panic！！！
  
你可以在/src/router下新建一个你自己的.rs文件，但有一些要求：
1. 文件名只能包含小写字母或数字或下划线还有固定的.rs后缀，严禁在任何文件名或者网址路径中出现空格！
2. 路由的规范命名是这样的： 平台小写英文名_功能小写英文名
3. 文件名的除了后缀名的部分就是你的路由名;
4. 每一个路由在/docs下都必须有一个对应为 路由名.md的文档，待会再讲;
5. 每一个路由只能对应一种功能;

开发前，最好先编译一次，增量编译在开发模式已经开启，第一次编译会耗时十分钟以内，要下很多的Crates,之后都是十秒钟编译完的事
### 导入库
```rust
use std::collections::HashMap;
use crate::easyuser::*;
use anyhow::*;
use rss::*;
```
上面这几个库**必须导入**，下面的而看需要导入
```rust
use serde_json::Value;//json用的
use scraper::*;//html用的
```
### 每个路由的主函数规范长这样：
```rust
pub async fn get(para: HashMap<String,String>) -> Result<String, Error> {
}
```
函数的调入和返回结果类型不能改的！（如果你没有用到 para,变量，你可以直接把他写成 _para ,这样编译器就不会警告）其中，para的作用是传入用户访问时的通过http get协议传入的参数，以及序列化成HashMap格式了。

注意签名里的 `async`：因为 `fetch_reqwest_get` 等抓取函数都是异步的，`get` 自己也必须是 `async` 才能 `await` 它们。

### 异步常见错误（写路由时最常踩的坑）

下面是新路由在 `cargo build` 时最常报的编译错误，按报错信息列出。遇到哪条照着改哪条。

**1. `await is only allowed inside async functions and blocks`（E0728）**

你在这个函数里写了 `.await`，但这个函数本身不是 `async`。最常见于：把抓详情页的逻辑拆成了一个普通辅助函数：

```
// 错误：helper 不是 async，却想 await
fn fetch_detail(url: &str) -> Result<String, Error> {
    fetch_reqwest_get(url).await?   // 编译报错 E0728
}
```
改成 `async fn`，并且调用处也要 `.await`：
```
async fn fetch_detail(url: &str) -> Result<String, Error> {
    fetch_reqwest_get(url).await?
}
let detail = fetch_detail(&url).await?;
```

**2. `Result<String, Error> is not a future`（E0277）**

忘了加 `.await`。`fetch_reqwest_get(url)` 返回的不是 `String`，而是一个"将来才拿到结果的 Future"；只有 `.await` 才能把它变成 `String`：
```rust
// 错误：拿到的是 Future，不是 String
let html = fetch_reqwest_get(&url)?;
// 正确：加 .await 才得到 String
let html = fetch_reqwest_get(&url).await?;
```
检查你文件里所有 `fetch_reqwest_get` / `fetch_reqwest_post` / `fetch_reqwest_get_with_headers` 调用，是否都跟了 `.await`。

**3. `future cannot be sent between threads safely`（E0277，scraper 专属）**

只有用了 `scraper` 解析 HTML 才会碰到。`Html`、`Selector`、`ElementRef` 这些类型**不是 `Send`**，不能带过 `.await`。典型场景：先解析出列表 `doc`，然后在循环里抓详情页：

```rust
// 错误：doc（Html 类型）活着穿过 .await，编译报 not Send
let doc = Html::parse_document(&html);
for link in &links {
    let detail_html = fetch_reqwest_get(link).await?;   // 报错：doc 在 await 那边还活着
}
```
修法是**"先解析、后抓取"两段式**：第一步先把列表页解析完，把需要的字段全存成 `String`/`Vec<String>`（在 `{ }` 块里用 `doc`，块结束就自动释放）；第二步才循环去抓详情页，此时手里只有纯数据：

```rust
// 第一步：解析列表，只收集 String。doc 在块结束就 drop 了
let items: Vec<(String, String)> = {
    let doc = Html::parse_document(&html);
    let sel = Selector::parse("a.title").unwrap();
    doc.select(&sel).map(|e| {
        let title = e.text().collect::<String>().trim().to_string();
        let href = e.value().attr("href").unwrap_or("").to_string();
        (title, href)
    }).collect()
};
// 第二步：此时没有 scraper 类型了，随便 await
for (title, href) in &items {
    let detail_html = fetch_reqwest_get(href).await?;
}
```

**4. 详情页也要用 scraper？同样先解析完再放下**

如果你抓完详情页还要再解析它、甚至再抓下一页，记住同一个原则：`detail_doc` 用完立刻丢弃。惯用写法是把详情解析放进 `if let Ok(...) = fetch(...).await { ... }` 块里，块内完成解析、块结束自动释放，块外不再引用它。

> 小结：**凡是要 `await`，身边就不要有 `Html`/`Selector`/`ElementRef`**。看到 `not Send` 就回想这一条。
>
> 参考实现：`chinanews`、`stcn_article_list`、`bjnews_cat`、`eastday_24`、`solidot`、`guancha_headline`、`guanhai`、`ithome_ranking`、`jianshu_home` 这些路由就是按上面的模式写的。

### 一个完整的最小路由（JSON 接口版）

下面是一个可以照抄的完整例子。它抓一个 JSON 接口，转成 RSS，保存为 `src/router/my_test.rs` 就能编译：

```rust
use crate::easyuser::*;          // 抓取函数、时间处理等都在这
use anyhow::{Error, Result};
use rss::*;                       // RSS 生成
use serde_json::Value;            // 解析 JSON 用
use std::collections::HashMap;

pub async fn get(para: HashMap<String,String>) -> Result<String, Error> {
    // 1. 取用户参数（可选，默认给一个）
    let id = para.get("id").cloned().unwrap_or_default();

    // 2. 抓上游。注意：fetch 是异步的，调用后要加 .await
    let html = fetch_reqwest_get(&format!("https://api.example.com/list?id={}", id)).await?;

    // 3. 解析 JSON
    let json: Value = serde_json::from_str(&html)?;
    let list = json["data"]["list"].as_array().ok_or_else(|| anyhow!("没有 list 字段"))?;

    // 4. 一条条转成 RSS Item
    let mut item_vec = Vec::new();
    for item in list {
        let rss_item = ItemBuilder::default()
            .title(Some(item["title"].as_str().unwrap_or("").to_string()))
            .link(Some(item["url"].as_str().unwrap_or("").to_string()))
            .description(item["desc"].as_str().unwrap_or(""))
            .pub_date(now())           // 不会解析时间就先填当前时间
            .build();
        item_vec.push(rss_item);
    }

    // 5. 组装成频道输出
    let channel = ChannelBuilder::default()
        .title("我的测试频道")
        .link("https://api.example.com")
        .description("示例")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
```
写好后按下面「注册」两步操作，`cargo build` 一下，跑起来访问 `/my_test` 就能看到你的 RSS 了。如果上游是 HTML 而非 JSON，把第 3 步换成 `scraper` 解析（注意上面第 3、4 条错误）。

项目封装了一些方便用户的函数，可以在src/easyuser.rs查看，或者在[easyuser_cn.html](easyuser_cn.md)查看
具体的程序逻辑就由你自己编写吧。
### 写完怎么注册呢？
在 `/src/router/mod.rs` 中，在最后新建一行，写上 `pub mod 你的路由名;`
然后再在 `src/request_rules.rs` 的 `match url` 分发中，仿照其他条目添加一个分支：
```rust
"/你的路由名" => run!(你的路由名, parameters),
```
`run!` 宏会展开为 `你的路由名::get(parameters.clone()).await`，因此不要改动它的写法。
> **注意**：`run!` 宏参数中的 `parameters` 是 `request_rules` 的入参名，宏内由于卫生性无法直接引用外层变量，所以必须显式传入。
### 注册完怎么运行呢？
我对二进制文件设置的一些**必要环境目录十分严苛**，`cookies.json`,`index`文件夹等，都必须在二进制文件同目录下，因此我创建了一个`env`文件夹，需要的环境都在里面。他应该是长这样的
```sh
├── cookies.json
├── docs_md
│   ├── .......md
│   └── official
│       └── ......md
├── index
│   ├── 404.html
│   └── index.html
└── rssust
```
我还写了一个脚本叫build.sh供用户使用（目前只做了Linux系统的脚本，Windows系统用户可以自己做一个脚本或者手动操作），其内容是 编译，拷贝（把/target/debug/rssust拷贝到/env，接着运行。
### Cookies的防止
在与二进制文件同目录下的`cookies.json`中如下：
 示例格式：
 {
   "name": "session_id",
   "value": "abc123",
   "domain": ".example.com",
   "path": "/",
   "secure": true,
   "httpOnly": true,
   "sameSite": "Lax",
   "expirationDate": 1893456000
 }
 所有的cookies挤在同一个json文件，程序启动时就会导入。
Tips：可以使用扩展 Cookie-Editor导出为json到剪贴板再合并。
### 文档的编写！
在/docs下新建 你的路由名.md的文件，复制粘贴一下再慢慢填：
```markdown
# Router-name: 
**Commit time:** 
**Cookies?:** 
**Author:** 
**Introduction:** 
**Address:** 
**Example:** 
**Parameter:** 
**Environment Variables:**
```
首先除了`**Cookies?:**`,`**Environment Variables:**`和`**Parameter:** `,其余基本上都是字面意思，但我还是介绍一下：
1. **Router-name:** 填路由名
2. **Commit time:** 提交时间，以`年.月.日`的形式填写
3. **Cookies?:** 两种情况，一种为`no`，一种为yes,后面也可以写备注解释
4. **Author:** 提交人，写你的Github名
5. **Introduction:** 写一下简介（注意，如果平台以什么语言为主就用什么语言）
6. **Address:** 填 `rssust://路由名`就好
7. **Example:** 可选项，如果要选就要用超链接`[填Address相同的](填/路由名)`
8. **Parameter:** 这个意思是http get时可能用到的参数，可以填`no`, 或者空着，另起一行，像下面的示例
``` markdown
**Parameter:** 
1. **disableembed** 
   Type of parameter: bool
   Default value: true
   Meaning: 内嵌视频
2. 接下来如法炮制
```
注意：必填的项目把默认值设为null
其中的参数名称跟在序号后面，用`** **`包裹强调
**Type of parameter:** 填类型，自己定
**Default value:** 默认值，要符合`Type of parameter`
**Meaning:** 参数的含义
9. **Environment Variables:** 可能用到的环境变量，填no,或者空着，另起一行，像下面的示例
``` markdown
**Parameter:** 
1. **disableembed** 
   Type of parameter: bool
   Default value: true
   Meaning: 内嵌视频
2. 接下来如法炮制
```
注意：必填的项目把默认值设为null，每个项目的意思参考**Parameter:** 的子项
### 最后一步：提交！
使用git提交前，先运行`cargo fmt`，这可以使别人更好的识别你的代码
你需要提交的有：
1. /src/router/你的路由名.rs
2. env/docs_md/路由名.md (一般情况下请不要提交您的`路由名.html`文件)
3. /src/router/mod.rs以及src/request_rules.rs
   
#### 这只是路由的指南，如果你想改进核心代码，欢迎提交关于代码的改进，完善
如果好奇程序构造的，可以看[rogram_explanation_cn](pogram_explanation_cn.md)