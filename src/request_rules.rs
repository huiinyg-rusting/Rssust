use crate::config::{is_route_disabled, rate_limit_secs};
use crate::router::*;
use anyhow::*;
use std::collections::HashMap;
use std::time::Duration;
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

macro_rules! run {
    ($route:ident, $params:expr) => {
        $route::get($params.clone()).await
    };
}

macro_rules! routes {
    ($(($route:literal, $module:ident)),* $(,)?) => {
        pub const ROUTES: &[&str] = &[$($route),*];
        pub const ROUTE_COUNT: usize = ROUTES.len();

        pub async fn route_dispatch(
            url: &str,
            parameters: HashMap<String, String>,
        ) -> Result<String, anyhow::Error> {
            match url {
                $($route => run!($module, parameters),)*
                _ => {
                    warn!("Unregistered route: {}", url);
                    Err(anyhow!("404NotFound"))
                }
            }
        }
    };
}

routes! {
    ("/apnews_topics", apnews_topics),
    ("/baidu_top", baidu_top),
    ("/bilibili_audio_rank", bilibili_audio_rank),
    ("/bilibili_bangumi_follow", bilibili_bangumi_follow),
    ("/bilibili_blackroom", bilibili_blackroom),
    ("/bilibili_collection", bilibili_collection),
    ("/bilibili_dynamic", bilibili_dynamic),
    ("/bilibili_fav", bilibili_fav),
    ("/bilibili_followers", bilibili_followers),
    ("/bilibili_link_news", bilibili_link_news),
    ("/bilibili_live_area", bilibili_live_area),
    ("/bilibili_partion", bilibili_partion),
    ("/bilibili_partion_ranking", bilibili_partion_ranking),
    ("/bilibili_popular", bilibili_popular),
    ("/bilibili_precious", bilibili_precious),
    ("/bilibili_search_hot", bilibili_search_hot),
    ("/bilibili_series", bilibili_series),
    ("/bilibili_splash", bilibili_splash),
    ("/bilibili_user_article", bilibili_user_article),
    ("/bilibili_user_coin", bilibili_user_coin),
    ("/bilibili_user_fav", bilibili_user_fav),
    ("/bilibili_user_like", bilibili_user_like),
    ("/bilibili_video_page", bilibili_video_page),
    ("/bilibili_video_reply", bilibili_video_reply),
    ("/bilibili_vsearch", bilibili_vsearch),
    ("/bilibili_weekly", bilibili_weekly),
    ("/bjnews_cat", bjnews_cat),
    ("/caixin_latest", caixin_latest),
    ("/carnegieendowment_news", carnegieendowment_news),
    ("/cenc_earthquake", cenc_earthquake),
    ("/chinanews", chinanews),
    ("/cls_hot", cls_hot),
    ("/crates_new", crates_new),
    ("/defensenews_news", defensenews_news),
    ("/defenseone_news", defenseone_news),
    ("/devto_guides", devto_guides),
    ("/discovermagazine_news", discovermagazine_news),
    ("/douban_book_latest", douban_book_latest),
    ("/douban_book_rank", douban_book_rank),
    ("/douban_event_hot", douban_event_hot),
    ("/douban_movie_classification", douban_movie_classification),
    ("/eastday_24", eastday_24),
    ("/eeo_kuaixun", eeo_kuaixun),
    ("/gelonghui_home", gelonghui_home),
    ("/github_advisor", github_advisor),
    ("/github_branch", github_branch),
    ("/github_commits", github_commits),
    ("/github_contributors", github_contributors),
    ("/github_discussions", github_discussions),
    ("/github_followers", github_followers),
    ("/github_gist", github_gist),
    ("/github_issue", github_issue),
    ("/github_issue_comments", github_issue_comments),
    ("/github_pull", github_pull),
    ("/github_release", github_release),
    ("/github_repo_events", github_repo_events),
    ("/github_repo_stargazers", github_repo_stargazers),
    ("/github_search", github_search),
    ("/github_starred", github_starred),
    ("/github_stars", github_stars),
    ("/github_tag", github_tag),
    ("/github_topic", github_topic),
    ("/github_trending", github_trending),
    ("/github_user_events", github_user_events),
    ("/github_user_repos", github_user_repos),
    ("/guancha_headline", guancha_headline),
    ("/guanhai", guanhai),
    ("/guokr_scientific", guokr_scientific),
    ("/hackernews", hackernews),
    ("/ifeng_news", ifeng_news),
    ("/ithome_ranking", ithome_ranking),
    ("/jianshu_home", jianshu_home),
    ("/juejin_pins", juejin_pins),
    ("/juejin_trending", juejin_trending),
    ("/kali_blog", kali_blog),
    ("/leiphone_newsflash", leiphone_newsflash),
    ("/lwn", lwn),
    ("/mittrchina", mittrchina),
    ("/moe_news", moe_news),
    ("/nasa_apod", nasa_apod),
    ("/netease_today", netease_today),
    ("/nmc_alarm", nmc_alarm),
    ("/openai_chatgpt_atlas_release", openai_chatgpt_atlas_release),
    ("/openai_chatgpt_release", openai_chatgpt_release),
    ("/openai_news", openai_news),
    ("/openai_research", openai_research),
    ("/pingwest_news", pingwest_news),
    ("/rail12306_news", rail12306_news),
    ("/rail12306_ticket", rail12306_ticket),
    ("/scientificamerican_news", scientificamerican_news),
    ("/sina_finance", sina_finance),
    ("/smithsonianmag_news", smithsonianmag_news),
    ("/solidot", solidot),
    ("/stcn_article_list", stcn_article_list),
    ("/stcn_kx", stcn_kx),
    ("/stcn_rank", stcn_rank),
    ("/thepaper_featured", thepaper_featured),
    ("/tmtpost_new", tmtpost_new),
    ("/toutiao_hot", toutiao_hot),
    ("/videocardz_news", videocardz_news),
    ("/wallstreetcn_hot", wallstreetcn_hot),
    ("/yicai_headline", yicai_headline),
    ("/yicai_latest", yicai_latest),
    ("/zhihu_daily", zhihu_daily),
    ("/zhihu_hot", zhihu_hot),
}

pub async fn request_rules(
    url: &str,
    parameters: HashMap<String, String>,
) -> Result<String, anyhow::Error> {
    if is_route_disabled(url) {
        warn!("Route {} is disabled", url);
        return Err(anyhow!("404NotFound"));
    }
    debug!("Route {} matched, fetching", url);
    let ttl = rate_limit_secs(url).map(Duration::from_secs);
    crate::rate_limit::with_cache_scope(ttl, async {
        let result: Result<String, anyhow::Error> = route_dispatch(url, parameters).await;
        match &result {
            std::result::Result::Ok(_) => debug!("Route {} generated successfully", url),
            std::result::Result::Err(e) => {
                warn!("Route {} generation failed: {}", url, e);
                crate::rate_limit::cleanup_on_error();
            }
        }
        result
    })
    .await
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
    } else if first_part == "/status" {
        let body = format!(
            "{{\"status\":\"ok\",\"uptime_secs\":{},\"routes\":{},\"cache_entries\":{},\"requests\":{},\"failures\":{}}}",
            crate::stats::up_secs(),
            ROUTE_COUNT,
            crate::rate_limit::len(),
            crate::stats::requests(),
            crate::stats::failures(),
        );
        ShowToUser::Html {
            res: std::result::Result::Ok(body),
        }
    } else {
        match request_rules(first_part, second_part).await {
            std::result::Result::Ok(i) => ShowToUser::Rss { res: Ok(i) },
            Err(i) => ShowToUser::Html { res: Err(i) },
        }
    }
}
