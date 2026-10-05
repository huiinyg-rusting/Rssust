/* rhai 脚本路由，仅在 feature "scripts" 下编译。
scripts/ 下平铺的 .rhai 文件名即路由名，静态路由未命中时兜底执行。
脚本不碰网络：//@source 由 Rust 侧抓取后注入 raw，脚本 return
[{ title, link, description, pub_date, author }] 数组。
脚本里的 let doc = `...` 是该路由的 markdown 文档，rssust docs 从中提取。
详见 env/docs_md/official/script_route_cn.md */

use crate::easyuser::{HttpError, fetch_reqwest_get};
use anyhow::{Error, Result, anyhow};
use rhai::{AST, Dynamic, Engine, Scope};
use rss::{ChannelBuilder, ItemBuilder};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime};
use tracing::{debug, warn};

/* 单个脚本路由的缓存条目：mtime 用于热重载判断，ast 为编译结果 */
#[derive(Clone)]
struct ScriptEntry {
    mtime: SystemTime,
    name: Option<String>,
    source: Option<String>,
    doc: Option<String>,
    ast: Option<AST>,
    compile_error: Option<String>,
}

struct ScriptTable {
    entries: HashMap<String, ScriptEntry>,
    last_scan: Option<Instant>,
}

const REFRESH_INTERVAL: Duration = Duration::from_secs(2);

fn scripts_root() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default()
        .join("scripts")
}

/* 收集 scripts/ 下的 .rhai 文件（不递归子目录，路由 = 文件名） */
fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            continue;
        }
        if p.extension().is_some_and(|x| x == "rhai") {
            out.push(p);
        }
    }
}

/* 文件名去 .rhai 即路由名，如 bilibili_partion_ranking_script.rhai → /bilibili_partion_ranking_script */
fn route_of(file: &Path) -> String {
    let stem = file
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    format!("/{}", stem)
}

/* 解析脚本头元数据注释与内嵌 doc 块：
//@name 频道标题 / //@source 抓取地址 / //@item CSS 选择器(预留) /
let doc = `...` 为该路由的 markdown 文档 */
fn parse_meta(
    content: &str,
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    let mut name = None;
    let mut source = None;
    let mut item = None;
    let mut doc = None;
    let mut collecting = false;
    let mut buf = String::new();
    for line in content.lines() {
        if collecting {
            if let Some(pos) = line.find('`') {
                buf.push_str(&line[..pos]);
                if !buf.trim().is_empty() {
                    doc = Some(buf.trim().to_string());
                }
                collecting = false;
            } else {
                buf.push_str(line);
                buf.push('\n');
            }
            continue;
        }
        let t = line.trim();
        if let Some(r) = t.strip_prefix("//@name ") {
            name = Some(r.trim().to_string());
        } else if let Some(r) = t.strip_prefix("//@source ") {
            source = Some(r.trim().to_string());
        } else if let Some(r) = t.strip_prefix("//@item ") {
            item = Some(r.trim().to_string());
        } else if let Some(pos) = t.find("let doc =") {
            let rest = t[pos + "let doc =".len()..].trim_start();
            if let Some(inner) = rest.strip_prefix('`') {
                if let Some(end) = inner.find('`') {
                    if !inner[..end].trim().is_empty() {
                        doc = Some(inner[..end].trim().to_string());
                    }
                } else {
                    buf.push_str(inner);
                    collecting = true;
                }
            }
        }
    }
    (name, source, item, doc)
}

/* 路由表：每 REFRESH_INTERVAL 重扫一次，mtime 变了才重编译 */
fn table_state() -> std::sync::MutexGuard<'static, ScriptTable> {
    static TABLE: OnceLock<Mutex<ScriptTable>> = OnceLock::new();
    let mut guard = TABLE
        .get_or_init(|| {
            Mutex::new(ScriptTable {
                entries: HashMap::new(),
                last_scan: None,
            })
        })
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    if guard.last_scan.is_none() || guard.last_scan.unwrap().elapsed().gt(&REFRESH_INTERVAL) {
        let root = scripts_root();
        let mut entries = HashMap::new();
        let mut files = Vec::new();
        walk(&root, &mut files);
        for f in files {
            let route = route_of(&f);
            let mtime = f
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            /* mtime 未变则沿用旧条目，含已编译的 AST */
            if let Some(old) = guard.entries.get(&route) {
                if old.mtime == mtime {
                    entries.insert(route, old.clone());
                    continue;
                }
            }
            let content = match std::fs::read_to_string(&f) {
                Ok(c) => c,
                Err(e) => {
                    warn!("脚本 {} 读取失败: {}", route, e);
                    continue;
                }
            };
            let (name, source, _item, doc) = parse_meta(&content);
            let (ast, compile_error) = match Engine::new().compile(&content) {
                Ok(a) => (Some(a), None),
                Err(e) => (None, Some(format!("脚本 {} 编译错误: {}", route, e))),
            };
            entries.insert(
                route.clone(),
                ScriptEntry {
                    mtime,
                    name,
                    source,
                    doc,
                    ast,
                    compile_error,
                },
            );
        }
        guard.entries = entries;
        guard.last_scan = Some(Instant::now());
    }
    guard
}

/* 脚本返回的对象转成字符串 map，字段缺失不报错 */
fn item_from_dynamic(v: Dynamic) -> Result<HashMap<String, String>, Error> {
    let m = v
        .try_cast::<rhai::Map>()
        .ok_or_else(|| anyhow!("脚本返回数组元素必须是对象（{{字段: 值}}）"))?;
    let mut out = HashMap::new();
    for k in ["title", "link", "description", "pub_date", "author"] {
        if let Some(val) = m.get(k) {
            let s = val
                .clone()
                .into_string()
                .unwrap_or_else(|_| val.to_string());
            out.insert(k.to_string(), s);
        }
    }
    Ok(out)
}

fn items_from_dynamic(out: Dynamic, route: &str) -> Result<Vec<HashMap<String, String>>, Error> {
    let arr = out
        .try_cast::<rhai::Array>()
        .ok_or_else(|| anyhow!("脚本 {} 必须 return 数组（条目列表）", route))?;
    let mut items = Vec::new();
    for v in arr {
        items.push(item_from_dynamic(v)?);
    }
    Ok(items)
}

/* 兜底入口：静态路由 404 时调用，抓取 → 执行 → 建 RSS */
pub async fn try_route(route: &str, params: HashMap<String, String>) -> Result<String, Error> {
    let entry = {
        let g = table_state();
        g.entries.get(route).cloned()
    };
    let entry = match entry {
        Some(e) => e,
        None => return Err(HttpError::not_found("404NotFound").into()),
    };
    if let Some(msg) = &entry.compile_error {
        return Err(anyhow!("{}", msg));
    }
    let Some(ast) = entry.ast.clone() else {
        return Err(anyhow!("脚本 {} 未编译", route));
    };
    let Some(template) = entry.source.clone() else {
        return Err(anyhow!("脚本 {} 缺少 //@source 声明", route));
    };
    /* {参数名} 占位替换为 URL 查询参数 */
    let mut url = template;
    for (k, v) in &params {
        url = url.replace(&format!("{{{}}}", k), v);
    }
    let body = match fetch_reqwest_get(&url).await {
        Ok(b) => b,
        Err(e) => return Err(anyhow!("抓取 {} 失败: {}", url, e)),
    };
    debug!("脚本路由 {} 抓取 {} 成功，{} 字节", route, url, body.len());

    let name = entry
        .name
        .clone()
        .unwrap_or_else(|| format!("脚本 {}", route));
    /* 频道简介取 doc 的首个非标题行 */
    let channel_desc = entry
        .doc
        .as_ref()
        .and_then(|d| {
            d.lines()
                .map(str::trim)
                .find(|l| !l.is_empty() && !l.starts_with('#'))
                .map(|l| l.to_string())
        })
        .unwrap_or_else(|| format!("rhai 脚本路由 {}", route));
    let channel_link = url.clone();
    let raw = body.clone();
    let params_c = params.clone();
    let route_c = route.to_string();

    /* rhai 引擎为同步执行：spawn_blocking + panic 兜底，避免拖垮 tokio worker */
    let exec = tokio::task::spawn_blocking(move || {
        let engine = Engine::new();
        let mut scope = Scope::new();
        scope.push("raw", raw);
        let pm: rhai::Map = params_c
            .iter()
            .map(|(k, v)| (k.as_str().into(), Dynamic::from(v.clone())))
            .collect();
        scope.push("params", pm);
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let out: Dynamic = engine
                .eval_ast_with_scope(&mut scope, &ast)
                .map_err(|e| anyhow!("脚本 {} 执行失败: {}", route_c, e))?;
            items_from_dynamic(out, &route_c)
        }))
        .map_err(|_| anyhow!("脚本 {} 执行 panic", route_c))?
    })
    .await;
    let items = exec??;

    let mut item_vec = Vec::new();
    for it in &items {
        let title = it.get("title").cloned().unwrap_or_default();
        let link = it.get("link").cloned().unwrap_or_default();
        if title.trim().is_empty() && link.trim().is_empty() {
            continue;
        }
        let mut description = String::new();
        if let Some(d) = it.get("description") {
            description = d.clone();
        }
        let mut pub_date = None;
        if let Some(p) = it.get("pub_date")
            && !p.trim().is_empty()
        {
            pub_date = Some(p.clone());
        }
        let mut author = None;
        if let Some(a) = it.get("author")
            && !a.trim().is_empty()
        {
            author = Some(a.clone());
        }
        item_vec.push(
            ItemBuilder::default()
                .title(Some(title))
                .link(Some(link))
                .description(description)
                .pub_date(pub_date)
                .author(author)
                .build(),
        );
    }
    let channel = ChannelBuilder::default()
        .title(name)
        .link(channel_link)
        .description(channel_desc)
        .items(item_vec)
        .build();
    Ok(channel.to_string())
}

/* 脚本路由是否已注册 */
pub fn has_route(route: &str) -> bool {
    table_state().entries.contains_key(route)
}

/* 脚本文档条目：路由名（不含 .md）、完整 markdown */
pub struct ScriptDoc {
    pub name: String,
    pub markdown: String,
}

/* 去掉 doc 内与标题重复的首行 # 标题，避免渲染出两个同名标题 */
fn strip_dup_title(doc_body: &str, title: &str) -> String {
    let mut seen = false;
    doc_body
        .lines()
        .filter(|l| {
            let t = l.trim();
            if !seen && t.starts_with('#') && t.trim_start_matches('#').trim() == title {
                seen = true;
                return false;
            }
            true
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/* 扫描 scripts/ 提取脚本内嵌 markdown（在 docs 生成阶段调用，不落 .md 文件） */
pub fn collect_script_docs() -> Vec<ScriptDoc> {
    let root = scripts_root();
    let mut files = Vec::new();
    walk(&root, &mut files);
    files.sort();
    let mut out = Vec::new();
    for f in files {
        let Ok(content) = std::fs::read_to_string(&f) else {
            continue;
        };
        let route = route_of(&f);
        let (name, source, _, doc) = parse_meta(&content);
        let stem = route.trim_start_matches('/').to_string();
        let title = name.unwrap_or_else(|| format!("脚本路由 {}", route));
        let doc_body = strip_dup_title(&doc.unwrap_or_default(), &title);
        /* 首个非标题行作为简介，同时从正文里摘掉，避免页面上重复 */
        let mut intro = String::new();
        let mut kept: Vec<&str> = Vec::new();
        for line in doc_body.lines() {
            let t = line.trim();
            if intro.is_empty() && !t.is_empty() && !t.starts_with('#') {
                intro = t.to_string();
                continue;
            }
            kept.push(line);
        }
        if intro.is_empty() {
            intro = "动态 rhai 脚本路由".to_string();
        }
        let body = kept.join("\n").trim().to_string();
        let mut markdown = String::new();
        markdown.push_str(&format!("# {}\n\n", title));
        markdown.push_str(&format!("**Introduction:** {}\n\n", intro));
        if !body.is_empty() {
            markdown.push_str(&body);
            markdown.push_str("\n\n");
        }
        markdown.push_str("---\n\n### 路由信息\n\n");
        markdown.push_str(&format!("- 路由路径: `{}`\n", route));
        if let Some(s) = &source {
            markdown.push_str(&format!("- 抓取地址: `{}`\n", s));
        }
        markdown.push_str("- 类型: rhai 脚本（动态路由，编译需开启 scripts feature）\n");
        markdown.push_str("- 脚本位置: exe 目录 `scripts/` 下\n\n");
        markdown
            .push_str("> 启用方式: `cargo build --features scripts` 编译后，脚本路由自动生效。\n");
        out.push(ScriptDoc {
            name: stem,
            markdown,
        });
    }
    out
}
