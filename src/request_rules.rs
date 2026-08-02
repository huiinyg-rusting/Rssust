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

///展开为 `mod::get(params).await` 的表达式。
///路由均为 `pub async fn get(HashMap<String,String>) -> Result<String>`，
///所以这里必须产出一个表达式，不能产出 match 分支。
macro_rules! run {
    ($route:ident, $params:expr) => {
        $route::get($params.clone()).await
    };
}

///这个函数相当于模块的注册表
/// 给调用者的是html格式
pub async fn request_rules(
    url: &str,
    parameters: HashMap<String, String>,
) -> Result<String, anyhow::Error> {
    if is_route_disabled(url) {
        warn!("Route {} is disabled", url);
        return Err(anyhow!("404NotFound"));
    }
    debug!("Route {} matched, fetching", url);
    let result: Result<String, anyhow::Error> = match url {
        "/apnews_topics" => run!(apnews_topics, parameters),
        "/bjnews_cat" => run!(bjnews_cat, parameters),
        "/bilibili_weekly" => run!(bilibili_weekly, parameters),
        "/bilibili_dynamic" => run!(bilibili_dynamic, parameters),
        "/bilibili_popular" => run!(bilibili_popular, parameters),
        "/bilibili_precious" => run!(bilibili_precious, parameters),
        "/bilibili_series" => run!(bilibili_series, parameters),
        "/bilibili_collection" => run!(bilibili_collection, parameters),
        "/bilibili_fav" => run!(bilibili_fav, parameters),
        "/bilibili_link_news" => run!(bilibili_link_news, parameters),
        "/bilibili_partion" => run!(bilibili_partion, parameters),
        "/bilibili_partion_ranking" => run!(bilibili_partion_ranking, parameters),
        "/bilibili_user_article" => run!(bilibili_user_article, parameters),
        "/bilibili_user_coin" => run!(bilibili_user_coin, parameters),
        "/bilibili_user_fav" => run!(bilibili_user_fav, parameters),
        "/bilibili_user_like" => run!(bilibili_user_like, parameters),
        "/bilibili_video_page" => run!(bilibili_video_page, parameters),
        "/bilibili_video_reply" => run!(bilibili_video_reply, parameters),
        "/bilibili_vsearch" => run!(bilibili_vsearch, parameters),
        "/douban_book_latest" => run!(douban_book_latest, parameters),
        "/douban_book_rank" => run!(douban_book_rank, parameters),
        "/douban_event_hot" => run!(douban_event_hot, parameters),
        "/douban_movie_classification" => run!(douban_movie_classification, parameters),
        "/defenseone_news" => run!(defenseone_news, parameters),
        "/defensenews_news" => run!(defensenews_news, parameters),
        "/discovermagazine_news" => run!(discovermagazine_news, parameters),
        "/eastday_24" => run!(eastday_24, parameters),
        "/eeo_kuaixun" => run!(eeo_kuaixun, parameters),
        "/extremetech_news" => run!(extremetech_news, parameters),
        "/carnegieendowment_news" => run!(carnegieendowment_news, parameters),
        "/netease_today" => run!(netease_today, parameters),
        "/gelonghui_home" => run!(gelonghui_home, parameters),
        "/hupu_news" => run!(hupu_news, parameters),
        "/thepaper_featured" => run!(thepaper_featured, parameters),
        "/leiphone_newsflash" => run!(leiphone_newsflash, parameters),
        "/nmc_alarm" => run!(nmc_alarm, parameters),
        "/solidot" => run!(solidot, parameters),
        "/smithsonianmag_news" => run!(smithsonianmag_news, parameters),
        "/scientificamerican_news" => run!(scientificamerican_news, parameters),
        "/stcn_article_list" => run!(stcn_article_list, parameters),
        "/stcn_kx" => run!(stcn_kx, parameters),
        "/stcn_rank" => run!(stcn_rank, parameters),
        "/wallstreetcn_hot" => run!(wallstreetcn_hot, parameters),
        "/caixin_latest" => run!(caixin_latest, parameters),
        "/chinanews" => run!(chinanews, parameters),
        "/cls_hot" => run!(cls_hot, parameters),
        "/ifeng_news" => run!(ifeng_news, parameters),
        "/guancha_headline" => run!(guancha_headline, parameters),
        "/guanhai" => run!(guanhai, parameters),
        "/ithome_ranking" => run!(ithome_ranking, parameters),
        "/jianshu_home" => run!(jianshu_home, parameters),
        "/juejin_pins" => run!(juejin_pins, parameters),
        "/juejin_trending" => run!(juejin_trending, parameters),
        "/yicai_latest" => run!(yicai_latest, parameters),
        "/yicai_headline" => run!(yicai_headline, parameters),
        "/tmtpost_new" => run!(tmtpost_new, parameters),
        "/videocardz_news" => run!(videocardz_news, parameters),
        "/zhihu_hot" => run!(zhihu_hot, parameters),
        _ => {
            warn!("Unregistered route: {}", url);
            return Err(anyhow!("404NotFound"));
        }
    };
    match &result {
        std::result::Result::Ok(_) => debug!("Route {} generated successfully", url),
        std::result::Result::Err(e) => warn!("Route {} generation failed: {}", url, e),
    }
    result
}

pub async fn root_rules(first_part: &str, second_part: HashMap<String, String>) -> ShowToUser {
    if first_part == "/" {
        ShowToUser::Html {
            res: crate::connect::show_index_doc().await,
        }
    } else if first_part == "/favicon.ico" {
        crate::connect::serve_static("/index/favicon.ico").await
    } else if first_part.starts_with("/docs/") || first_part.starts_with("/index/") {
        crate::connect::serve_static(first_part).await
    } else {
        match request_rules(first_part, second_part).await {
            std::result::Result::Ok(i) => ShowToUser::Rss { res: Ok(i) },
            Err(i) => ShowToUser::Html { res: Err(i) },
        }
    }
}
