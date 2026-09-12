use anyhow::Error;
use mdbook::MDBook;
use mdbook::config::Config;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

fn parse_api_groups(api_path: &Path) -> (Vec<String>, HashMap<String, String>) {
    let mut groups = Vec::new();
    let mut map = HashMap::new();
    let content = match std::fs::read_to_string(api_path) {
        Ok(s) => s,
        Err(_) => return (groups, map),
    };
    let mut current = String::new();
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("#### ") {
            current = rest.to_string();
            if !groups.contains(&current) {
                groups.push(current.clone());
            }
        } else if let Some(rest) = line.strip_prefix("- [")
            && let Some(name) = rest.split(']').next()
            && !current.is_empty()
        {
            map.insert(name.to_string(), current.clone());
        }
    }
    (groups, map)
}

fn md_intro(path: &Path) -> String {
    let content = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return String::new(),
    };
    for line in content.lines() {
        if let Some(rest) = line.trim().strip_prefix("**Introduction:**") {
            return rest.trim().to_string();
        }
    }
    String::new()
}

fn collect_route_names(source_root: &Path) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let entries = match std::fs::read_dir(source_root) {
        Ok(e) => e,
        Err(_) => return names,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "md") {
            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            if name != "SUMMARY" {
                names.insert(name);
            }
        }
    }
    names
}

fn generate_api_file(source_root: &Path) -> Result<(), Error> {
    let api_path = source_root.join("official/api.md");
    let (_, group_map) = parse_api_groups(&api_path);

    let mut grouped: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for name in collect_route_names(source_root) {
        let intro = md_intro(&source_root.join(format!("{}.md", name)));
        let group = group_map
            .get(&name)
            .cloned()
            .unwrap_or_else(|| "Other".to_string());
        grouped.entry(group).or_default().push((name, intro));
    }

    let mut out = String::new();
    out.push_str("### The API provided by this server\n");
    out.push_str("### 本服务器所提供的API\n");
    out.push_str("---\n");
    for (group, routes) in &grouped {
        out.push_str(&format!("#### {}\n", group));
        for (name, intro) in routes {
            out.push_str(&format!("- [{}](../{}.md)\n", name, name));
            if intro.is_empty() {
                out.push_str("    (缺简介)\n");
            } else {
                out.push_str(&format!("    {}\n", intro));
            }
        }
        out.push('\n');
    }
    std::fs::write(&api_path, out)?;
    Ok(())
}

fn routes_from_mod(src_root: &Path) -> BTreeSet<String> {
    let path = src_root.join("router/mod.rs");
    let content = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return BTreeSet::new(),
    };
    let mut set = BTreeSet::new();
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("pub mod ") {
            let name = rest.trim_end_matches(';').trim().to_string();
            if !name.is_empty() {
                set.insert(name);
            }
        }
    }
    set
}

fn routes_from_routes_macro(src_root: &Path) -> BTreeSet<String> {
    let path = src_root.join("request_rules.rs");
    let content = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return BTreeSet::new(),
    };
    let mut set = BTreeSet::new();
    let mut in_block = false;
    for line in content.lines() {
        let t = line.trim();
        if !in_block {
            if t.starts_with("routes!") {
                in_block = true;
            }
            continue;
        }
        if t.ends_with('}') {
            break;
        }
        if let Some(rel) = t.find("\"/") {
            let rest = &t[rel + 1..];
            if let Some(end) = rest.find('"') {
                set.insert(rest[1..end].to_string());
            }
        }
    }
    set
}

fn check_route_consistency(exe_dir: &Path) -> Vec<String> {
    let src_root = exe_dir.parent().unwrap_or(exe_dir).join("src");
    let doc_routes = collect_route_names(&exe_dir.join("docs_md"));
    let mod_routes = routes_from_mod(&src_root);
    let request_routes = routes_from_routes_macro(&src_root);

    if mod_routes.is_empty() {
        return vec!["无法读取 src/router/mod.rs，跳过 mod 对账".to_string()];
    }
    if request_routes.is_empty() {
        return vec!["无法读取 src/request_rules.rs，跳过 request_rules 对账".to_string()];
    }

    let mut issues: Vec<String> = Vec::new();
    for name in request_routes.difference(&mod_routes) {
        issues.push(format!(
            "路由 {} 已在 request_rules 注册但缺 router 模块声明",
            name
        ));
    }
    for name in request_routes.difference(&doc_routes) {
        issues.push(format!(
            "路由 {} 已在 request_rules 注册但缺文档 docs_md/{}.md",
            name, name
        ));
    }
    for name in doc_routes.difference(&request_routes) {
        issues.push(format!("文档 {} 存在但路由未在 request_rules 注册", name));
    }
    issues
}

pub fn doc_generate() -> Result<(), Error> {
    let exe_dir = std::env::current_exe()?.parent().unwrap().to_path_buf();
    let source_root = exe_dir.join("docs_md");
    let official_dir = source_root.join("official");
    let build_dir = exe_dir.join("docs");

    // 生成 SUMMARY.md
    let mut summary = String::new();
    summary.push_str("# 官方文档\n");
    let mut official_names: Vec<String> = Vec::new();
    if official_dir.is_dir() {
        for entry in std::fs::read_dir(&official_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "md") {
                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                official_names.push(name);
            }
        }
    }
    official_names.sort();
    for name in &official_names {
        summary.push_str(&format!("- [{}](official/{}.md)\n", name, name));
    }

    summary.push_str("\n# 路由\n");
    let mut route_names: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&source_root)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "md") {
            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            if name != "SUMMARY" {
                route_names.push(name);
            }
        }
    }
    route_names.sort();
    for name in &route_names {
        let file = format!("{}.md", name);
        summary.push_str(&format!("- [{}]({})\n", name, file));
    }

    std::fs::write(source_root.join("SUMMARY.md"), summary)?;

    // 自动生成 official/api.md（路由清单 + Introduction 描述）
    generate_api_file(&source_root)?;

    // 三方对账：mod.rs / request_rules.rs / docs_md
    let issues = check_route_consistency(&exe_dir);
    let consistency_ok = issues.is_empty();
    for issue in &issues {
        tracing::error!("DOC-ISS: {}", issue);
    }

    // 构建
    let mut config = Config::default();
    config.set("book.src", ".")?;
    config.set("output.html.site-url", "/docs/")?;
    config.set("output.html.no-section-label", true)?;
    config.set("output.html.curly-quotes", true)?;
    config.set("build.build-dir", build_dir.to_string_lossy())?;

    let book = MDBook::load_with_config(source_root, config)?;
    book.build()?;

    // 将 official/ 下的 HTML 文件移动到 docs/ 根目录
    let official_out = build_dir.join("official");
    if official_out.is_dir() {
        for entry in std::fs::read_dir(&official_out)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "html") {
                let dest = build_dir.join(path.file_name().unwrap());
                std::fs::rename(&path, &dest)?;
            }
        }
        std::fs::remove_dir(&official_out)?;
    }

    // 修复所有 HTML 和 JS 文件中的链接
    // 注意：不直接替换所有 href="../"→href=""，因为 API 示例链接
    // (如 ../bilibili_dynamic?uid=...) 需要保留 ../ 以从 /docs/ 回到 / 路由
    let fix_content = |content: String| -> String {
        let mut s = content;
        // toc.js / toc.html / nav 中的 official/ 前缀
        s = s.replace("href=\"official/", "href=\"");
        // 脚本和 iframe 的 src
        s = s.replace("src=\"../", "src=\"");
        // former-official 文件中的 ../official/xxx 链接
        s = s.replace("href=\"../official/", "href=\"");
        // path_to_root
        s = s.replace("path_to_root = \"../\"", "path_to_root = \"\"");
        s = s.replace(
            "path_to_searchindex_js = \"../searchindex.js\"",
            "path_to_searchindex_js = \"searchindex.js\"",
        );
        // 资源文件：css, FontAwesome, fonts, favicon
        s = s.replace("href=\"../css/", "href=\"css/");
        s = s.replace("href=\"../FontAwesome/", "href=\"FontAwesome/");
        s = s.replace("href=\"../fonts/", "href=\"fonts/");
        s = s.replace("href=\"../favicon", "href=\"favicon");
        // 已知资源文件
        for file in &[
            "highlight.css",
            "tomorrow-night.css",
            "ayu-highlight.css",
            "elasticlunr.min.js",
            "mark.min.js",
            "searcher.js",
            "clipboard.min.js",
            "highlight.js",
            "book.js",
            "print.html",
            "toc.html",
        ] {
            s = s.replace(
                &format!("href=\"../{}", file),
                &format!("href=\"{}\"", file),
            );
        }
        // 其他 HTML 页面链接 (xxx.html) — 这些是文档间跳转，不是 API 示例
        // 移除 href="../xxx.html" 中的 "../"，保留 API 路由链接的 "../"
        let needle = "href=\"../";
        let mut result = String::with_capacity(s.len());
        let mut pos = 0;
        while let Some(found) = s[pos..].find(needle) {
            result.push_str(&s[pos..pos + found]);
            let val_start = pos + found + 9; // after href="../
            if let Some(end) = s[val_start..].find('"') {
                let target = &s[val_start..val_start + end];
                if target.ends_with(".html") || target.starts_with("print.html") {
                    result.push_str("href=\"");
                    result.push_str(target);
                    result.push('"');
                } else {
                    result.push_str("href=\"../");
                    result.push_str(target);
                    result.push('"');
                }
                pos = val_start + end + 1;
            } else {
                result.push_str(&s[pos + found..]);
                pos = s.len();
                break;
            }
        }
        result.push_str(&s[pos..]);
        result
    };

    for entry in std::fs::read_dir(&build_dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension()
            && (ext == "html" || ext == "js")
        {
            let content = std::fs::read_to_string(&path)?;
            let content = fix_content(content);
            std::fs::write(&path, content)?;
        }
    }

    if consistency_ok {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "文档三方对账发现 {} 个问题（见上方 DOC-ISS 日志）",
            issues.len()
        ))
    }
}
