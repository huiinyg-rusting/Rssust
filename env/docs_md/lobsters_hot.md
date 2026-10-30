# Router-name: lobsters_hot
**Commit time:** 2026.10.25
**Cookies?:** no
**Author:** AI（基于 RSSHub /lobste.rs 系列自行实现）
**Introduction:** lobste.rs 热门/最新文章（via public JSON API: /hottest.json, /newest.json, /t/{tag}.json），含分数、评论数、标签与摘要
**Address:** rssust://lobsters_hot
**Example:** [rssust://lobsters_hot](/lobsters_hot)
**robots.txt:** lobste.rs 的 robots.txt 设置了 Crawl-delay: 30，建议配置限流 600s
**Parameter:**
1. **sort**
   Type of parameter: string
   Default value: hot
   Meaning: hot=热门（/hottest.json），newest=最新（/newest.json），带 tag 时仅支持 hot
2. **tag**
   Type of parameter: string
   Default value: null（不筛标签）
   Meaning: 只取某标签下的文章，如 rust / security
3. **page**
   Type of parameter: number
   Default value: 1
   Meaning: 分页页码（上游每页约 25 条）
**Environment Variables:** no
