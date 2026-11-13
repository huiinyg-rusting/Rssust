# Router-name: douyin_hot  
**Commit time:** 2026.11.07  
**Cookies?:** no  
**Author:** AI（自行实现，RSSHub 无对应路由）  
**Introduction:** 抖音热搜榜，数据来自抖音网页端公开 JSON API；含热度值、上榜时间与话题封面图  
**Address:** rssust://douyin_hot  
**Example:** [rssust://douyin_hot](/douyin_hot)  
**robots.txt:** douyin.com 的 robots.txt 没有针对未知抓取器的通配组（无 `User-agent: *`），热搜接口路径不受限制；建议限流 300s  
**Parameter:**  
1. **limit**  
   Type of parameter: number  
   Default value: 10  
   Meaning: 返回条数，最大 50（上游榜单一次约 50 条）  
**Environment Variables:** no