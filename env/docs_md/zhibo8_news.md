# Router-name: zhibo8_news
**Commit time:** 2026.10.31
**Cookies?:** no
**Author:** AI（自行实现，RSSHub 无对应路由）
**Introduction:** 直播吧体育资讯，支持 NBA / 足球 / 电竞等栏目；自动去重并按链接取前 N 条，可选抓取详情页正文
**Address:** rssust://zhibo8_news
**Example:** [rssust://zhibo8_news](/zhibo8_news)
**robots.txt:** news.zhibo8.com 无 robots.txt（404），无抓取限制；建议限流 300s
**Parameter:**
1. **channel**
   Type of parameter: string
   Default value: nba
   Meaning: 栏目，可选 nba（篮球）/ zuqiu（足球）/ dianjing（电竞）/ other
2. **limit**
   Type of parameter: number
   Default value: 20
   Meaning: 返回条数，最大 50
3. **detail**
   Type of parameter: bool
   Default value: true
   Meaning: 是否抓详情页正文与精确发布时间；关闭后只输出标题和列表页日期，速度更快
**Environment Variables:** no
