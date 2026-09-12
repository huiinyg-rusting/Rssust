### The API provided by this server
### 本服务器所提供的API
---
#### AP News
- [apnews_topics](../apnews_topics.md)
    AP News headlines by topic

#### BJNews (新京报)
- [bjnews_cat](../bjnews_cat.md)
    新京报分类文章

#### Baidu (百度热搜)
- [baidu_top](../baidu_top.md)
    百度热搜榜单（实时/科技/娱乐等分类），数据取自 top.baidu.com 页面内嵌 JSON，无官方 RSS

#### Bilibili
- [bilibili_audio_rank](../bilibili_audio_rank.md)
    B站音频榜单最新一期（热榜/原创榜）
- [bilibili_bangumi_follow](../bilibili_bangumi_follow.md)
    用户追番（追剧）列表，含总集数、最新一话与更新时间
- [bilibili_blackroom](../bilibili_blackroom.md)
    B站小黑屋封禁公示列表
- [bilibili_collection](../bilibili_collection.md)
    B站合集（合集，不是系列）视频列表，通过 season_id 获取合集内的视频
- [bilibili_dynamic](../bilibili_dynamic.md)
    获取指定 Bilibili 用户的动态（综合、视频、图文、直播、转发等），生成 RSS 源。入口函数 `pub async fn get(para: HashMap<String, String>) -> Result<String, Error>`，位于 `src/router/bilibili_dynamic.rs`。内部调用 Bilibili 官方 API `/x/polymer/web-dynamic/v1/feed/space`。
- [bilibili_fav](../bilibili_fav.md)
    bilibili UP主非默认收藏夹
- [bilibili_followers](../bilibili_followers.md)
    某 UP 主的粉丝列表
- [bilibili_link_news](../bilibili_link_news.md)
    bilibili 直播/小视频/相簿公告
- [bilibili_live_area](../bilibili_live_area.md)
    B站直播分区 (某分区下房间列表)
- [bilibili_partion](../bilibili_partion.md)
    bilibili 分区视频
- [bilibili_partion_ranking](../bilibili_partion_ranking.md)
    bilibili 分区视频排行榜
- [bilibili_popular](../bilibili_popular.md)
    B站热门视频
- [bilibili_precious](../bilibili_precious.md)
    B站入站必刷宝藏视频，从 api.bilibili.com/x/web-interface/popular/precious 获取
- [bilibili_search_hot](../bilibili_search_hot.md)
    B站热搜榜前10关键词
- [bilibili_series](../bilibili_series.md)
    B站系列（！！！只能系列，不能合集）视频列表，通过 series_id 获取系列中的视频
- [bilibili_splash](../bilibili_splash.md)
    B站APP端开屏广告信息
- [bilibili_user_article](../bilibili_user_article.md)
    bilibili UP 主图文(专栏)
- [bilibili_user_coin](../bilibili_user_coin.md)
    bilibili UP 主投币视频
- [bilibili_user_fav](../bilibili_user_fav.md)
    bilibili UP 主默认收藏夹
- [bilibili_user_like](../bilibili_user_like.md)
    bilibili UP 主点赞视频
- [bilibili_video_page](../bilibili_video_page.md)
    bilibili 视频选集列表
- [bilibili_video_reply](../bilibili_video_reply.md)
    bilibili 视频评论
- [bilibili_vsearch](../bilibili_vsearch.md)
    bilibili 视频搜索
- [bilibili_weekly](../bilibili_weekly.md)
    Bilibili最新一期的每周精选

#### CENC (中国地震台网中心)
- [cenc_earthquake](../cenc_earthquake.md)
    中国地震台网中心 最新地震速报

#### Cailianpress (财联社)
- [cls_hot](../cls_hot.md)
    财联社热门文章排行榜

#### Caixin (财新网)
- [caixin_latest](../caixin_latest.md)
    财新网最新文章

#### Carnegie Endowment
- [carnegieendowment_news](../carnegieendowment_news.md)
    Carnegie Endowment for International Peace - International affairs think tank publications, events & videos (no official RSS)

#### Chinanews (中国新闻网)
- [chinanews](../chinanews.md)
    中国新闻网滚动新闻

#### DEV.to
- [devto_guides](../devto_guides.md)
    DEV.to Trending Guides (official RSS feed tag=guides)

#### Defense News
- [defensenews_news](../defensenews_news.md)
    Defense News - Global Defense & Military News (no official RSS, scraped)

#### Defense One
- [defenseone_news](../defenseone_news.md)
    Defense One - Defense & National Security News (no official RSS)

#### Discover Magazine
- [discovermagazine_news](../discovermagazine_news.md)
    Discover Magazine - Science News (no official RSS)

#### Douban
- [douban_book_latest](../douban_book_latest.md)
    豆瓣新书速递
- [douban_book_rank](../douban_book_rank.md)
    豆瓣热门图书排行
- [douban_event_hot](../douban_event_hot.md)
    豆瓣同城热门活动
- [douban_movie_classification](../douban_movie_classification.md)
    豆瓣电影分类

#### EEO (经济观察报)
- [eeo_kuaixun](../eeo_kuaixun.md)
    经济观察报快讯

#### Eastday (东方资讯)
- [eastday_24](../eastday_24.md)
    东方资讯24小时热闻

#### Gelonghui (格隆汇)
- [gelonghui_home](../gelonghui_home.md)
    格隆汇首页，支持分类（推荐/股票/基金/新股/研报）

#### GitHub
- [github_advisor](../github_advisor.md)
    GitHub Advisory Database Security Advisories (REST API)
- [github_branch](../github_branch.md)
    GitHub 仓库分支列表
- [github_commits](../github_commits.md)
    GitHub repository recent commits on default branch via GraphQL API
- [github_contributors](../github_contributors.md)
    GitHub 仓库贡献者列表
- [github_discussions](../github_discussions.md)
    GitHub 仓库 Discussion 讨论列表 (GraphQL)
- [github_followers](../github_followers.md)
    GitHub user followers query via GraphQL API
- [github_gist](../github_gist.md)
    用户公开/私密 Gist 列表
- [github_issue](../github_issue.md)
    GitHub 仓库 Issue 列表
- [github_issue_comments](../github_issue_comments.md)
    GitHub repository recent Issue / Pull Request comments via GraphQL API
- [github_pull](../github_pull.md)
    GitHub 仓库 Pull Request 列表
- [github_release](../github_release.md)
    GitHub 仓库 Release 列表
- [github_repo_events](../github_repo_events.md)
    GitHub 仓库事件流 (push/issue/PR/release/star 等)
- [github_repo_stargazers](../github_repo_stargazers.md)
    GitHub 仓库加星用户时间线（含 Star 时间）
- [github_search](../github_search.md)
    GitHub 仓库搜索
- [github_starred](../github_starred.md)
    GitHub 用户收藏的仓库列表
- [github_stars](../github_stars.md)
    GitHub single repository star count via GraphQL API
- [github_tag](../github_tag.md)
    GitHub 仓库 Tag 列表
- [github_topic](../github_topic.md)
    GitHub Topics 话题下的仓库列表（抓取网页）
- [github_trending](../github_trending.md)
    GitHub Trending repositories (no official RSS, scraped)
- [github_user_events](../github_user_events.md)
    GitHub 用户最近公开动态（events）
- [github_user_repos](../github_user_repos.md)
    GitHub 用户的仓库列表

#### Guancha (观察者网)
- [guancha_headline](../guancha_headline.md)
    观察者网头条

#### Guanhai (观海新闻)
- [guanhai](../guanhai.md)
    观海新闻首页推荐

#### Guokr (果壳网)
- [guokr_scientific](../guokr_scientific.md)
    果壳网 科学人 文章

#### Hacker News
- [hackernews](../hackernews.md)
    Hacker News Stories (official Algolia API)

#### IT之家
- [ithome_ranking](../ithome_ranking.md)
    IT之家热榜

#### Jianshu (简书)
- [jianshu_home](../jianshu_home.md)
    简书首页

#### Juejin (掘金)
- [juejin_pins](../juejin_pins.md)
    掘金沸点
- [juejin_trending](../juejin_trending.md)
    掘金热门文章

#### Kali Linux
- [kali_blog](../kali_blog.md)
    Kali Linux 官方博客（blog.kali.org 的更新，含发布日志与工具介绍）

#### LWN
- [lwn](../lwn.md)
    LWN.net（Linux 内核与技术新闻）官方 RSS 转接

#### Leiphone (雷锋网)
- [leiphone_newsflash](../leiphone_newsflash.md)
    雷锋网业界资讯

#### MIT Technology Review (麻省理工科技评论)
- [mittrchina](../mittrchina.md)
    MIT Technology Review 麻省理工科技评论中文站

#### MOE (教育部)
- [moe_news](../moe_news.md)
    教育部司局要闻列表（服务端渲染 HTML，无官方 RSS）

#### NASA
- [nasa_apod](../nasa_apod.md)
    NASA 每日天文一图（APOD），官方公开 API，无官方 RSS

#### NMC (中央气象台)
- [nmc_alarm](../nmc_alarm.md)
    中央气象台预警信号

#### NetEase
- [netease_today](../netease_today.md)
    网易新闻今日关注

#### OpenAI
- [openai_chatgpt_atlas_release](../openai_chatgpt_atlas_release.md)
    ChatGPT Atlas Release Notes (help.openai.com single page scrape)
- [openai_chatgpt_release](../openai_chatgpt_release.md)
    ChatGPT Release Notes (help.openai.com single page scrape)
- [openai_news](../openai_news.md)
    OpenAI Official News (rss.xml + full article content)
- [openai_research](../openai_research.md)
    OpenAI Research Articles (filtered from official RSS by category=Research)

#### PingWest (品玩)
- [pingwest_news](../pingwest_news.md)
    品玩 PingWest 首页精选资讯（无官方 RSS，抓取首页静态区块）

#### Rail12306 (中国铁路12306)
- [rail12306_news](../rail12306_news.md)
    12306 最新动态公告
- [rail12306_ticket](../rail12306_ticket.md)
    12306 火车票余票查询

#### STCN (证券时报网)
- [stcn_article_list](../stcn_article_list.md)
    证券时报网文章列表，按分类（要闻/股市/公司/基金/金融/评论/产经/科创板/新三板/ESG/滚动）
- [stcn_kx](../stcn_kx.md)
    证券时报网快讯
- [stcn_rank](../stcn_rank.md)
    证券时报网热榜（按分类）

#### Scientific American
- [scientificamerican_news](../scientificamerican_news.md)
    Scientific American - Science & Technology News (no official RSS)

#### Sina (新浪)
- [sina_finance](../sina_finance.md)
    新浪财经 7×24 直播快讯（公开 JSON API，无官方 RSS）

#### Smithsonian Magazine
- [smithsonianmag_news](../smithsonianmag_news.md)
    Smithsonian Magazine - Science, History & Culture (official RSS returns 403)

#### Solidot
- [solidot](../solidot.md)
    奇客的资讯，重要的东西

#### TMTpost (钛媒体)
- [tmtpost_new](../tmtpost_new.md)
    钛媒体最新文章

#### ThePaper
- [thepaper_featured](../thepaper_featured.md)
    澎湃新闻首页头条推荐

#### Toutiao (今日头条)
- [toutiao_hot](../toutiao_hot.md)
    今日头条热榜（公开 JSON API，无官方 RSS）

#### VideoCardz
- [videocardz_news](../videocardz_news.md)
    VideoCardz.com - Latest GPU & Hardware News (English tech site, no official RSS)

#### WallStreetCN
- [wallstreetcn_hot](../wallstreetcn_hot.md)
    华尔街见闻最热文章

#### Yicai (第一财经)
- [yicai_headline](../yicai_headline.md)
    第一财经头条
- [yicai_latest](../yicai_latest.md)
    第一财经最新文章

#### Zhihu
- [zhihu_daily](../zhihu_daily.md)
    知乎日报（官方公开 API，无官方 RSS）
- [zhihu_hot](../zhihu_hot.md)
    知乎热榜

#### crates.io
- [crates_new](../crates_new.md)
    crates.io 最新发布的 Rust crate 聚合（官方公开 API，无官方 RSS）

#### iFeng (凤凰网)
- [ifeng_news](../ifeng_news.md)
    凤凰网资讯

