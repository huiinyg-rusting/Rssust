### [中文](new_router_cn.md)
# Router Development Guide
This document is for both humans and AI to read.

Before you start developing, you need:
- A standard Cargo environment
- Git version control system
- An editor
- **Basic** Rust knowledge, preferably through the official book, and familiarity with frequently used methods in the std library, such as various error handling methods for recoverable errors, etc.
- RSS knowledge from: [Official RSS 2.0.11 English](https://www.rssboard.org/rss-specification) or [Blog Garden - Translated by Xiao Y RSS 2.0.10 Chinese](https://www.cnblogs.com/tuyile006/p/3691024.html)
- Knowledge of RSS Crates, understanding how to build an RSS feed. [Official English Documentation](https://docs.rs/rss/latest/rss/)
  **Especially** [ChannelBuilder](https://docs.rs/rss/latest/rss/struct.ChannelBuilder.html) and [ImageBuilder](https://docs.rs/rss/latest/rss/struct.ImageBuilder.html)
- Since different upstream APIs have different formats, you need to learn different methods. For example, if your upstream API is in JSON format, you likely only need to learn the `.pointer()` and `.get()` methods from the serde_json Crate. If you need to directly process HTML and DOM, you need the methods recommended in the [scraper](https://docs.rs/scraper/0.27.0/scraper/) documentation, and be careful to avoid causing panic!!!

You can create your own .rs file under `/src/router`, but there are some requirements:
1. Filenames can only contain lowercase letters, numbers, underscores, and the fixed .rs extension. Absolutely no spaces in any filename or URL path!
2. Router naming convention is: `platform_lowercase_english_name_function_lowercase_english_name`
3. The filename part (excluding the .rs extension) becomes your router name
4. Each router must have a corresponding documentation file named `router_name.md` under `/docs`; more details later
5. Each router can only correspond to one function

Before developing, it's best to compile once first. Incremental compilation is already enabled in development mode. The first compilation will take less than ten minutes to download many Crates, and subsequent compilations will complete in about ten seconds.

### Import Libraries
```rust
use std::collections::HashMap;
use crate::easyuser::*;
use anyhow::*;
use rss::*;
```
The above libraries **must be imported**. Import the following as needed:
```rust
use serde_json::Value;  // for JSON
use scraper::*;  // for HTML
```

### The main function signature for each router should look like this:
```rust
pub async fn get(para: HashMap<String,String>) -> Result<String, Error> {
}
```
The function signature and return type cannot be changed! (If you don't use the `para` variable, you can write it as `_para` so the compiler won't warn you). The purpose of `para` is to receive parameters passed via HTTP GET protocol when users access the router, serialized into HashMap format.

Note the `async` keyword: since the fetching helpers (`fetch_reqwest_get`, etc.) are async, `get` itself must be `async` so it can `.await` them.

### Common async errors (the pitfalls you'll actually hit)

These are the compile errors you're most likely to see in a new router, listed by error message. Find yours and fix it.

**1. `await is only allowed inside async functions and blocks` (E0728)**

You wrote `.await` in a function that is not `async`. This most often happens when you extract the "fetch detail page" logic into a plain helper function:

```rust
// wrong: helper is not async but tries to await
fn fetch_detail(url: &str) -> Result<String, Error> {
    fetch_reqwest_get(url).await?   // E0728
}
```
Make it `async fn` — and `.await` the call site too:
```rust
async fn fetch_detail(url: &str) -> Result<String, Error> {
    fetch_reqwest_get(url).await?
}
let detail = fetch_detail(&url).await?;
```

**2. `Result<String, Error> is not a future` (E0277)**

You forgot `.await`. `fetch_reqwest_get(url)` does not return a `String` — it returns a Future that *will* become a `String` later. Only `.await` turns it into one:
```rust
// wrong: you got a Future, not a String
let html = fetch_reqwest_get(&url)?;
// right: add .await to get the String
let html = fetch_reqwest_get(&url).await?;
```
Audit every `fetch_reqwest_get` / `fetch_reqwest_post` / `fetch_reqwest_get_with_headers` call in your file and make sure each one has `.await`.

**3. `future cannot be sent between threads safely` (E0277, scraper only)**

You only hit this when parsing HTML with `scraper`. `Html`, `Selector`, and `ElementRef` are **not `Send`**, so they can't survive across an `.await`. Typical case: you parse a list into `doc`, then loop over links fetching detail pages:

```rust
// wrong: doc (an Html) is still alive across the .await
let doc = Html::parse_document(&html);
for link in &links {
    let detail_html = fetch_reqwest_get(link).await?;   // error: doc is still alive here
}
```
Fix it with a **two-phase "parse first, fetch later"** pattern. Step one: fully parse the list page, collecting every field into `String`/`Vec<String>` inside a `{ }` block (so `doc` drops at the block end). Step two: loop and fetch — now you only hold plain data:

```rust
// step 1: parse the list, collect only Strings. doc drops at the end of the block
let items: Vec<(String, String)> = {
    let doc = Html::parse_document(&html);
    let sel = Selector::parse("a.title").unwrap();
    doc.select(&sel).map(|e| {
        let title = e.text().collect::<String>().trim().to_string();
        let href = e.value().attr("href").unwrap_or("").to_string();
        (title, href)
    }).collect()
};
// step 2: no scraper types around, feel free to await
for (title, href) in &items {
    let detail_html = fetch_reqwest_get(href).await?;
}
```

**4. Need scraper for the detail page too? Same rule: parse, then let it go.**

If you fetch a detail page and then need to parse it (or even fetch the next page), keep the same principle: drop `detail_doc` as soon as you're done with it. The idiomatic way is to parse inside `if let Ok(...) = fetch(...).await { ... }` — the doc is created, used, and released inside that block, and never referenced outside it.

> TL;DR: **whenever you `.await`, don't hold `Html`/`Selector`/`ElementRef`**. If you see `not Send`, remember this.
>
> Reference implementations: `chinanews`, `stcn_article_list`, `bjnews_cat`, `eastday_24`, `solidot`, `guancha_headline`, `guanhai`, `ithome_ranking`, `jianshu_home` follow this pattern.

### A complete minimal router (JSON endpoint version)

Here's a full example you can copy. It fetches a JSON endpoint and turns it into RSS. Save it as `src/router/my_test.rs` and it compiles as-is:

```rust
use crate::easyuser::*;          // fetching helpers, date utils, etc.
use anyhow::{Error, Result};
use rss::*;                       // RSS generation
use serde_json::Value;            // JSON parsing
use std::collections::HashMap;

pub async fn get(para: HashMap<String,String>) -> Result<String, Error> {
    // 1. read user params (optional, with a default)
    let id = para.get("id").cloned().unwrap_or_default();

    // 2. fetch upstream. Note: fetch is async, add .await
    let html = fetch_reqwest_get(&format!("https://api.example.com/list?id={}", id)).await?;

    // 3. parse JSON
    let json: Value = serde_json::from_str(&html)?;
    let list = json["data"]["list"].as_array().ok_or_else(|| anyhow!("no list field"))?;

    // 4. convert each entry into an RSS Item
    let mut item_vec = Vec::new();
    for item in list {
        let rss_item = ItemBuilder::default()
            .title(Some(item["title"].as_str().unwrap_or("").to_string()))
            .link(Some(item["url"].as_str().unwrap_or("").to_string()))
            .description(item["desc"].as_str().unwrap_or(""))
            .pub_date(now())           // use now() if you can't parse the date yet
            .build();
        item_vec.push(rss_item);
    }

    // 5. assemble the channel
    let channel = ChannelBuilder::default()
        .title("My test channel")
        .link("https://api.example.com")
        .description("example")
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}
```
After writing it, follow the two registration steps below, run `cargo build`, start the server and visit `/my_test` to see your RSS. If your upstream serves HTML instead of JSON, replace step 3 with `scraper` parsing (watch out for errors 3 and 4 above).

The project has encapsulated some convenient functions for users, which you can check in `src/easyuser.rs` or view at [easyuser_cn.html](easyuser_cn.md).

The specific program logic is up to you to write.

### How to register it after writing?
In `/src/router/mod.rs`, add a new line at the end: `pub mod your_router_name;`

Then in `src/request_rules.rs`, add a match arm to the `match url` dispatcher, following the pattern of other entries:
```rust
"/your_router_name" => run!(your_router_name, parameters),
```
The `run!` macro expands to `your_router_name::get(parameters.clone()).await`, so don't change its shape.
> **Note**: `parameters` in the `run!` macro is the argument name of `request_rules`. Due to macro hygiene it can't refer to the outer variable, so it must be passed explicitly.

### How to run after registration?
I'm very strict about the necessary environment directories for the binary file. Files like `cookies.json`, the `index` folder, etc., must be in the same directory as the binary file. Therefore, I created an `env` folder where all necessary environments are located. It should look like this:
```sh
├── cookies.json
├── docs_md
│   ├── .......md
│   └── official
│       └── ......md
├── index
│   ├── 404.html
│   └── index.html
└── rssust
```

I've also written a script called `build.sh` for users (currently only Linux scripts available; Windows users can write their own script or do it manually). Its content: compile, copy (copy `/target/debug/rssust` to `/env`), then run.

### Cookie Management
In `cookies.json` in the same directory as the binary file, use the following format:

Example format:
```json
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
```
All cookies are in the same JSON file, which will be imported when the program starts.

Tips: You can use the Cookie-Editor extension to export as JSON to clipboard and then merge.

### Documentation Writing!
Create a file named `your_router_name.md` under `/docs`, copy and paste the template below and fill it in:
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

First, except for `**Cookies?:**`, `**Environment Variables:**`, and `**Parameter:**`, the rest are mostly self-explanatory, but I'll introduce them anyway:

1. **Router-name:** Fill in the router name
2. **Commit time:** Submission time, formatted as `year.month.day`
3. **Cookies?:** Two cases: `no` or `yes`, with optional remarks after
4. **Author:** Submitter, write your GitHub username
5. **Introduction:** Write a brief introduction (note: if the platform is mainly in a certain language, use that language)
6. **Address:** Fill in `rssust://router_name`
7. **Example:** Optional; if included, use a hyperlink: `[fill Address same as above](/router_name)`
8. **Parameter:** This means HTTP GET parameters that might be used. You can fill in `no`, leave it empty, or start a new line with the following format:
```markdown
**Parameter:** 
1. **disableembed** 
   Type of parameter: bool
   Default value: true
   Meaning: Embedded videos
2. Continue similarly for other parameters
```
The parameter name follows the number, wrapped with `** **` for emphasis.

**Type of parameter:** Fill in the type, define it yourself
**Default value:** Default value, must conform to the `Type of parameter`
**Meaning:** The meaning of the parameter

### Final Step: Submit!
Before submitting with git, run `cargo fmt` first. This helps others better understand your code.

You need to submit:
1. `/src/router/your_router_name.rs`
2. `env/docs_md/router_name.md` (generally please do not submit your `router_name.html` file)
3. `/src/router/mod.rs` and `src/request_rules.rs`

#### This is just a guide for routers. If you want to improve the core code, feel free to submit improvements to the code!
If you are insterested in pogram explanation,please turn to [pogram_explanation_cn](pogram_explanation_cn.md),but now it only has Chinese.