use anyhow::Error;
use mdbook::MDBook;
use mdbook::config::Config;

pub fn doc_generate() -> Result<(), Error> {
    let exe_dir = std::env::current_exe()?
        .parent()
        .unwrap()
        .to_path_buf();
    let source_root = exe_dir.join("docs_md");
    let official_dir = source_root.join("official");

    // 生成 SUMMARY.md 到 docs_md 根目录
    let mut summary = String::new();
    summary.push_str("# 官方文档\n");
    if official_dir.is_dir() {
        for entry in std::fs::read_dir(&official_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "md") {
                let name = path.file_stem().unwrap().to_string_lossy();
                summary.push_str(&format!("- [{}](official/{})\n", name, path.file_name().unwrap().to_string_lossy()));
            }
        }
    }

    summary.push_str("\n# 路由\n");
    for entry in std::fs::read_dir(&source_root)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy();
        if file_name == "official" || file_name == "SUMMARY.md" {
            continue;
        }
        if path.extension().map_or(false, |e| e == "md") {
            let name = path.file_stem().unwrap().to_string_lossy();
            summary.push_str(&format!("- [{}]({})\n", name, file_name));
        }
    }

    std::fs::write(source_root.join("SUMMARY.md"), summary)?;

    // 合并所有配置
    let mut config = Config::default();
    config.set("book.src", ".")?;
    config.set("output.html.site-url", "/docs/")?;
    config.set("output.html.no-section-label", "true")?;
    config.set("output.html.curly-quotes", "true")?;
    
    let build_dir = exe_dir.join("docs");
    config.set("build.build-dir", build_dir.to_string_lossy())?;

    let book = MDBook::load_with_config(source_root, config)?;
    book.build()?;

    Ok(())
}
