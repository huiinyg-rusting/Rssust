# Router-name: epicgames_free  
**Commit time:** 2026.10.31  
**Cookies?:** no  
**Author:** AI转写 / huiinyg审核 - 来源于RSShub@DIYgod  
**Introduction:** Epic 游戏商城限时免费游戏，当前可领的排在前面，并可选附带“即将免费”预告；数据来自 Epic 商店前端公开 JSON 接口  
**Address:** rssust://epicgames_free  
**Example:** [rssust://epicgames_free](/epicgames_free)  
**robots.txt:** 接口域名 store-site-backend-static.ak.epicgames.com 无 robots.txt，无抓取限制；建议限流 1800s（每周更新一次即可）  
**Parameter:**  
1. **locale**  
   Type of parameter: string  
   Default value: zh-CN  
   Meaning: 商店语言/区域（如 en-US），影响游戏名与链接语言  
2. **country**  
   Type of parameter: string  
   Default value: CN  
   Meaning: 国家代码，影响价格与可领范围  
3. **upcoming**  
   Type of parameter: bool  
   Default value: true  
   Meaning: 是否包含“即将免费”的游戏（标题带 [即将免费] 前缀）  
4. **limit**  
   Type of parameter: number  
   Default value: 20  
   Meaning: 返回条数上限  
**Environment Variables:** no
