use std::collections::HashMap;

fn test_route(name: &str, f: fn(HashMap<String, String>) -> Result<String, anyhow::Error>, params: HashMap<String, String>) {
    match f(params) {
        Ok(xml) => {
            let title = xml.lines().find(|l| l.contains("<title>") && !l.contains("</channel>")).unwrap_or("?").trim();
            let item_count = xml.matches("<item>").count();
            println!("{} — {} — {} items", name, title, item_count);
        }
        Err(e) => println!("{} — ERROR: {}", name, e),
    }
}

#[test]
fn print_all() {
    test_route("yicai_latest",         rssust::router::yicai_latest::get,         HashMap::new());
    test_route("yicai_headline",       rssust::router::yicai_headline::get,       HashMap::new());
    test_route("stcn_article_list",    rssust::router::stcn_article_list::get,    HashMap::new());
    test_route("stcn_kx",              rssust::router::stcn_kx::get,              HashMap::new());
    test_route("stcn_rank",            rssust::router::stcn_rank::get,            HashMap::new());
    test_route("leiphone_newsflash",   rssust::router::leiphone_newsflash::get,   HashMap::new());
    test_route("tmtpost_new",          rssust::router::tmtpost_new::get,          HashMap::new());
    test_route("gelonghui_home",       rssust::router::gelonghui_home::get,       HashMap::new());
}
