use anyhow::Error;
use mdbook::MDBook;
use mdbook::config::Config;

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
            if path.extension().map_or(false, |e| e == "md") {
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
    // 已定义的排序（Bilibili → CDE → Zhihu）
    let route_order = &[
        "bilibili_collection",
        "bilibili_dynamic",
        "bilibili_fav",
        "bilibili_link_news",
        "bilibili_partion",
        "bilibili_partion_ranking",
        "bilibili_popular",
        "bilibili_precious",
        "bilibili_series",
        "bilibili_user_article",
        "bilibili_user_coin",
        "bilibili_user_fav",
        "bilibili_user_like",
        "bilibili_video_page",
        "bilibili_video_reply",
        "bilibili_vsearch",
        "bilibili_weekly",
        "cde",
        "zhihu_hot",
    ];
    for name in route_order {
        let file = format!("{}.md", name);
        if source_root.join(&file).exists() {
            summary.push_str(&format!("- [{}]({})\n", name, file));
        }
    }
    // 未识别的 md 文件（避免遗漏）
    for entry in std::fs::read_dir(&source_root)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_stem().unwrap().to_string_lossy().to_string();
        if file_name == "official" || file_name == "SUMMARY" || route_order.contains(&file_name.as_str()) {
            continue;
        }
        if path.extension().map_or(false, |e| e == "md") {
            summary.push_str(&format!("- [{}]({})\n", file_name, path.file_name().unwrap().to_string_lossy()));
        }
    }

    std::fs::write(source_root.join("SUMMARY.md"), summary)?;

    // 构建
    let mut config = Config::default();
    config.set("book.src", ".")?;
    config.set("output.html.site-url", "/docs/")?;
    config.set("output.html.no-section-label", "true")?;
    config.set("output.html.curly-quotes", "true")?;
    config.set("build.build-dir", build_dir.to_string_lossy())?;

    let book = MDBook::load_with_config(source_root, config)?;
    book.build()?;

    // 将 official/ 下的 HTML 文件移动到 docs/ 根目录
    let official_out = build_dir.join("official");
    if official_out.is_dir() {
        for entry in std::fs::read_dir(&official_out)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "html") {
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
        if let Some(ext) = path.extension() {
            if ext == "html" || ext == "js" {
                let content = std::fs::read_to_string(&path)?;
                let content = fix_content(content);
                std::fs::write(&path, content)?;
            }
        }
    }

    Ok(())
}
