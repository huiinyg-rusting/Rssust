# Router-name: openai_news  
**Commit time:** 2026.08.06  
**Cookies?:** no  
**平台限制:** 文章详情抓取需 `curl-impersonate`（chrome110 TLS 指纹）。Windows x86_64 无预编译二进制 → 需从源码编译或使用 WSL；此路由失败时降级为 RSS 摘要（不 500）。参考 `openai_common.rs::fetch_article_details`  
**Author:** AI-converted / huiinyg-reviewed - based on RSSHub  
**Introduction:** OpenAI Official News (rss.xml + full article content)  
**Address:** rssust://openai_news  
**Example:** [rssust://openai_news](/openai_news)  
**Parameter:**  
    limit (optional, max 30, default 10)