use crate::config::is_route_disabled;
use crate::router::*;
use anyhow::*;
use std::collections::HashMap;
use tracing::{debug, warn};

pub enum ShowToUser {
    Html {
        res: Result<String, Error>,
    },
    Rss {
        res: Result<String, Error>,
    },
    File {
        res: Result<Vec<u8>, Error>,
        content_type: String,
    },
}

///这个函数相当于模块的注册表
/// 给调用者的是html格式
pub fn request_rules(
    url: &str,
    parameters: HashMap<String, String>,
) -> Result<String, anyhow::Error> {
    // 静态路由映射表，避免运行时重复构建
    const ROUTES: &[(
        &str,
        fn(HashMap<String, String>) -> Result<String, anyhow::Error>,
    )] = &[
        ("/apnews_topics", apnews_topics::get),
        ("/bjnews_cat", bjnews_cat::get),
        ("/bilibili_weekly", bilibili_weekly::get),
        ("/bilibili_dynamic", bilibili_dynamic::get),
        ("/bilibili_popular", bilibili_popular::get),
        ("/bilibili_precious", bilibili_precious::get),
        ("/bilibili_series", bilibili_series::get),
        ("/bilibili_collection", bilibili_collection::get),
        ("/bilibili_fav", bilibili_fav::get),
        ("/bilibili_link_news", bilibili_link_news::get),
        ("/bilibili_partion", bilibili_partion::get),
        ("/bilibili_partion_ranking", bilibili_partion_ranking::get),
        ("/bilibili_user_article", bilibili_user_article::get),
        ("/bilibili_user_coin", bilibili_user_coin::get),
        ("/bilibili_user_fav", bilibili_user_fav::get),
        ("/bilibili_user_like", bilibili_user_like::get),
        ("/bilibili_video_page", bilibili_video_page::get),
        ("/bilibili_video_reply", bilibili_video_reply::get),
        ("/bilibili_vsearch", bilibili_vsearch::get),
        ("/douban_book_latest", douban_book_latest::get),
        ("/douban_book_rank", douban_book_rank::get),
        ("/douban_event_hot", douban_event_hot::get),
        (
            "/douban_movie_classification",
            douban_movie_classification::get,
        ),
        ("/eastday_24", eastday_24::get),
        ("/eeo_kuaixun", eeo_kuaixun::get),
        ("/netease_today", netease_today::get),
        ("/gelonghui_home", gelonghui_home::get),
        ("/hupu_news", hupu_news::get),
        ("/thepaper_featured", thepaper_featured::get),
        ("/leiphone_newsflash", leiphone_newsflash::get),
        ("/nmc_alarm", nmc_alarm::get),
        ("/solidot", solidot::get),
        ("/stcn_article_list", stcn_article_list::get),
        ("/stcn_kx", stcn_kx::get),
        ("/stcn_rank", stcn_rank::get),
        ("/wallstreetcn_hot", wallstreetcn_hot::get),
        ("/caixin_latest", caixin_latest::get),
        ("/chinanews", chinanews::get),
        ("/cls_hot", cls_hot::get),
        ("/ifeng_news", ifeng_news::get),
        ("/guancha_headline", guancha_headline::get),
        ("/guanhai", guanhai::get),
        ("/ithome_ranking", ithome_ranking::get),
        ("/jianshu_home", jianshu_home::get),
        ("/juejin_pins", juejin_pins::get),
        ("/juejin_trending", juejin_trending::get),
        ("/yicai_latest", yicai_latest::get),
        ("/yicai_headline", yicai_headline::get),
        ("/tmtpost_new", tmtpost_new::get),
        ("/zhihu_hot", zhihu_hot::get),
    ];

    // 查找匹配的路由处理器
    if let Some(handler) = ROUTES.iter().find(|(path, _)| *path == url) {
        if is_route_disabled(url) {
            warn!("Route {} is disabled", url);
            return Err(anyhow!("404NotFound"));
        }
        debug!("Route {} matched, fetching", url);
        let result = (handler.1)(parameters);
        match &result {
            std::result::Result::Ok(_) => debug!("Route {} generated successfully", url),
            std::result::Result::Err(e) => warn!("Route {} generation failed: {}", url, e),
        }
        result
    } else {
        warn!("Unregistered route: {}", url);
        Err(anyhow!("404NotFound"))
    }
}
pub fn root_rules(first_part: &str, second_part: HashMap<String, String>) -> ShowToUser {
    if first_part == "/" {
        ShowToUser::Html {
            res: crate::connect::show_index_doc(),
        }
    } else if first_part == "/favicon.ico" {
        crate::connect::serve_static("/index/favicon.ico")
    } else if first_part.starts_with("/docs/") || first_part.starts_with("/index/") {
        crate::connect::serve_static(first_part)
    } else {
        match request_rules(first_part, second_part) {
            std::result::Result::Ok(i) => ShowToUser::Rss { res: Ok(i) },
            Err(i) => ShowToUser::Html { res: Err(i) },
        }
    }
}
