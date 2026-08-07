# Router-name: hackernews  
**Commit time:** 2026.08.06  
**Cookies?:** no  
**Author:** AI-converted / huinyg-reviewed - based on RSSHub  
**Introduction:** Hacker News Stories (official Algolia API)  
**Address:** rssust://hackernews  
**Example:** [rssust://hackernews](/hackernews)  
**Parameter:**  
    section (optional, index/newest/ask/show/jobs/best/over, default index)  
    type (optional, sources=story links comments=with comment summary, default sources)  
    value (optional, section=over for min points threshold default 100; others can append author=xxx)  
    limit (optional, max 50, default 30)  
**Rate limit:**  
    Recommend config "/hackernews" = 30 in config.toml routes.rate_limit (respect robots Crawl-delay: 30)