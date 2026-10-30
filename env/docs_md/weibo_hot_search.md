# Router-name: weibo_hot_search
**Commit time:** 2026.10.25
**Cookies?:** no（上游为公开 JSON 接口 ajax/side/hotSearch，但建议配置限流防风控）
**Author:** AI（基于 RSSHub /weibo/search/hot 自行实现）
**Introduction:** 微博热搜榜，实时热搜前 50 条，含火热值与榜单标签，可直接订阅
**Address:** rssust://weibo_hot_search
**Example:** [rssust://weibo_hot_search](/weibo_hot_search)
**robots.txt:** weibo.com 的 robots.txt 未禁止 /ajax/ 路径的获取
**Parameter:**
1. **limit**
   Type of parameter: number
   Default value: 50
   Meaning: 返回条数上限（上游一次返回 50 条）
2. **hotgov**
   Type of parameter: bool
   Default value: false
   Meaning: 是否在最前附带“置顶”话题条目
**Environment Variables:** no
