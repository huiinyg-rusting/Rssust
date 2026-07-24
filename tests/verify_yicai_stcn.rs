use std::collections::HashMap;

#[test]
fn test_yicai_latest() {
    let result = rssust::router::yicai_latest::get(HashMap::new()).unwrap();
    assert!(result.starts_with("<?xml"), "不是有效的 RSS XML: {}", &result[..100]);
    assert!(result.contains("第一财经"), "缺少站点名");
    assert!(result.contains("<item>"), "缺少 item");
    println!("yicai_latest OK");
}

#[test]
fn test_yicai_headline() {
    let result = rssust::router::yicai_headline::get(HashMap::new()).unwrap();
    assert!(result.starts_with("<?xml"));
    assert!(result.contains("第一财经"));
    assert!(result.contains("<item>"));
    println!("yicai_headline OK");
}

#[test]
fn test_stcn_article_list() {
    let result = rssust::router::stcn_article_list::get(HashMap::new()).unwrap();
    assert!(result.starts_with("<?xml"));
    assert!(result.contains("证券时报"));
    assert!(result.contains("<item>"));
    println!("stcn_article_list OK");
}

#[test]
fn test_stcn_kx() {
    let result = rssust::router::stcn_kx::get(HashMap::new()).unwrap();
    assert!(result.starts_with("<?xml"));
    assert!(result.contains("证券时报"));
    assert!(result.contains("<item>"));
    println!("stcn_kx OK");
}

#[test]
fn test_stcn_rank() {
    let result = rssust::router::stcn_rank::get(HashMap::new()).unwrap();
    assert!(result.starts_with("<?xml"));
    assert!(result.contains("证券时报"));
    assert!(result.contains("<item>"));
    println!("stcn_rank OK");
}

#[test]
fn test_leiphone_newsflash() {
    let result = rssust::router::leiphone_newsflash::get(HashMap::new()).unwrap();
    assert!(result.starts_with("<?xml"));
    assert!(result.contains("雷锋网"));
    assert!(result.contains("<item>"));
    println!("leiphone_newsflash OK");
}

#[test]
fn test_tmtpost_new() {
    let result = rssust::router::tmtpost_new::get(HashMap::new()).unwrap();
    assert!(result.starts_with("<?xml"));
    assert!(result.contains("钛媒体"));
    assert!(result.contains("<item>"));
    println!("tmtpost_new OK");
}

#[test]
fn test_gelonghui_home() {
    let result = rssust::router::gelonghui_home::get(HashMap::new()).unwrap();
    assert!(result.starts_with("<?xml"));
    assert!(result.contains("格隆汇"));
    assert!(result.contains("<item>"));
    println!("gelonghui_home OK");
}
