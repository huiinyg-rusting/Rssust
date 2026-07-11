### [中文](/docs/new_router_cn.html)
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
pub fn get(para: HashMap<String,String>) -> Result<String, Error> {
}
```
The function signature and return type cannot be changed! (If you don't use the `para` variable, you can write it as `_para` so the compiler won't warn you). The purpose of `para` is to receive parameters passed via HTTP GET protocol when users access the router, serialized into HashMap format.

The project has encapsulated some convenient functions for users, which you can check in `src/easyuser.rs` or view at [easyuser_cn.html](/docs/easyuser_cn.html).

The specific program logic is up to you to write.

### How to register it after writing?
In `/src/router/mod.rs`, add a new line at the end: `pub mod your_router_name;`

Then in the `request_rules()` function in `src/request_rules.rs`, add an `else if` branch with the condition `url == "/router_name"` (follow the pattern of other branches), and call your function in the `{}` block. Generally, the path is `your_router_name::get(parameters)`.

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