# Router-name: nasa_apod  
**Commit time:** 2026.08.30  
**Cookies?:** no  
**Author:** AI制作 / huiinyg-rusting审核  
**Introduction:** NASA 每日天文一图（APOD），官方公开 API，无官方 RSS  
**Address:** rssust://nasa_apod  
**Example:** [rssust://nasa_apod](/nasa_apod)  
**robots.txt:** api.nasa.gov 无 robots.txt，允许抓取；DEMO_KEY 限流约 30 次/小时 
**Parameter:**  
1. **count**  
   Type of parameter: int  
   Default value: 5  
   Meaning: 返回的图片条目数（最近若干天），范围 1-30  
**Environment Variables:** no