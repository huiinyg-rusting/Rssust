use crate::config::{is_route_disabled, rate_limit_secs};
use crate::easyuser::HttpError;
use crate::router::*;
use anyhow::*;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, warn};

/* ShowToUser：路由处理完的“响应载体”，由 connect::render 按类型决定 Content-Type
   - Html: 文本（首页/文档页/错误正文/status JSON）
   - Rss:  路由生成的 RSS XML
   - File: 二进制静态文件（css/js/图片/字体等），需附带 content_type */
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

/* run!：把 ("/xxx", xxx) 条目展开为 路由模块::get(参数).await 的调用动作 */
macro_rules! run {
    ($route:ident, $params:expr) => {
        $route::get($params.clone()).await
    };
}

/* routes!：路由注册表宏。列出 ("/路径", 模块名) 后自动生成：
   - ROUTES（路径数组）/ ROUTE_COUNT（总数）
   - route_dispatch()：match url 分发到对应路由模块
   新增路由只需在下方 routes!{...} 块按字母序加一行，无需手写 match */
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
                    Err(HttpError::not_found("404NotFound").into())
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
    ("/cas_research", cas_research),
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
    ("/gamersky_news", gamersky_news),
    ("/guokr_scientific", guokr_scientific),
    ("/hackernews", hackernews),
    ("/ifeng_news", ifeng_news),
    ("/ithome_ranking", ithome_ranking),
    ("/jianshu_home", jianshu_home),
    ("/jiemian_news", jiemian_news),
    ("/juejin_pins", juejin_pins),
    ("/juejin_trending", juejin_trending),
    ("/kali_blog", kali_blog),
    ("/leiphone_newsflash", leiphone_newsflash),
    ("/lwn", lwn),
    ("/mittrchina", mittrchina),
    ("/moe_news", moe_news),
    ("/mydrivers_news", mydrivers_news),
    ("/nasa_apod", nasa_apod),
    ("/nbd_news", nbd_news),
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

/* 二级分发：路由前置检查 + 限速缓存。
   1. routes.disabled 命中 → 404
   2. routes.rate_limit 配置了间隔 → 该请求进入缓存作用域（间隔内复用缓存，失败自动清理）
   3. 交给 route_dispatch 匹配具体路由模块 */
pub async fn request_rules(
    url: &str,
    parameters: HashMap<String, String>,
) -> Result<String, anyhow::Error> {
    if is_route_disabled(url) {
        warn!("Route {} is disabled", url);
        return Err(HttpError::not_found("404NotFound").into());
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

/* root_rules / request_rules 名字相近，职责区分：
   - root_rules：一级分发（HTTP 请求进来最先调用的入口）
   - request_rules：二级分发（路由前置检查 + 限速缓存 + 交给 route_dispatch） */
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
        /* /status 默认关闭，需在 config.toml 的 [server] 显式设置 status_route = true */
        if !crate::config::status_route_enabled() {
            return ShowToUser::Html {
                res: Err(HttpError::not_found("404NotFound").into()),
            };
        }
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
