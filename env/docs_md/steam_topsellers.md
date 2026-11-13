# Router-name: steam_topsellers  
**Commit time:** 2026.11.07  
**Cookies?:** no  
**Author:** AI（自行实现，RSSHub 无对应路由）  
**Introduction:** Steam 全球热销榜，含封面、当前价格与折扣信息；与 steam_discounts 同接口族（featuredcategories 的 top_sellers 区），数据来自 Steam 官方公开 JSON  
**Address:** rssust://steam_topsellers  
**Example:** [rssust://steam_topsellers](/steam_topsellers)  
**robots.txt:** store.steampowered.com 的 robots.txt 未禁止 /api/；建议限流 600s  
**Parameter:**  
1. **cc**  
   Type of parameter: string  
   Default value: cn  
   Meaning: 国家/地区代码，价格与货币随地区变化（如 us / jp / hk）  
2. **lang**  
   Type of parameter: string  
   Default value: schinese  
   Meaning: 语言代码（页面文字，不影响价格）  
3. **limit**  
   Type of parameter: number  
   Default value: 20  
   Meaning: 返回条数，最大 30（上游接口一次约 30 条）  
**Environment Variables:** no