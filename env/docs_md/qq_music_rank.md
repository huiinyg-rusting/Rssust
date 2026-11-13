# Router-name: qq_music_rank  
**Commit time:** 2026.11.07  
**Cookies?:** no  
**Author:** AI（自行实现，RSSHub 无对应路由）  
**Introduction:** QQ 音乐排行榜，支持巅峰/地区/特色/全球四组榜单（热歌榜、飙升榜、新歌榜、流行指数榜、内地榜、欧美榜等），含排名、歌手、专辑与时长；数据来自网页端 musicu.fcg 公开接口  
**Address:** rssust://qq_music_rank  
**Example:** [rssust://qq_music_rank](/qq_music_rank)  
**robots.txt:** u.y.qq.com 无 robots.txt（404，视为允许）；建议限流 600s（榜单每日更新一次）  
**Parameter:**  
1. **id**  
   Type of parameter: number  
   Default value: 26  
   Meaning: 榜单 topId。常用：26 热歌榜 / 62 飙升榜 / 27 新歌榜 / 4 流行指数榜 / 5 内地榜 / 3 欧美榜 / 16 韩国榜 / 17 日本榜 / 60 抖音热歌榜  
2. **limit**  
   Type of parameter: number  
   Default value: 30  
   Meaning: 返回条数，最大 100（榜单本身约 100~300 首）  
**Environment Variables:** no