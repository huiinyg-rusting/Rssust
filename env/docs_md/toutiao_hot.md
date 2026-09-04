# Router-name: toutiao_hot  
**Commit time:** 2026.08.31  
**Cookies?:** no  
**Author:** AI制作 / huiinyg-rusting审核  
**Introduction:** 今日头条热榜（公开 JSON API，无官方 RSS）  
**Address:** rssust://toutiao_hot  
**Example:** [rssust://toutiao_hot](/toutiao_hot)  
**robots.txt:** 允许抓取；toutiao.com 的 robots 仅 Disallow /search、/item、/group、/m1-m8、/trending/ 等路径，本路由只请求 `/hot-event/`  
**Parameter:**  
1. **count**  
   Type of parameter: int  
   Default value: 50  
   Meaning: 返回热搜条目数，范围 1-50  
**Environment Variables:** no