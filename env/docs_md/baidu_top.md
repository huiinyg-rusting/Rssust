# Router-name: baidu_top  
**Commit time:** 2026.08.30  
**Cookies?:** no  
**Author:** AI制作 / huiinyg-rusting审核  
**Introduction:** 百度热搜榜单（实时/科技/娱乐等分类），数据取自 top.baidu.com 页面内嵌 JSON，无官方 RSS  
**Address:** rssust://baidu_top  
**Example:** [rssust://baidu_top](/baidu_top)  
**robots.txt:** top.baidu.com 无 robots.txt，视为允许  
**Parameter:**  
1. **tab**  
   Type of parameter: string  
   Default value: realtime  
   Meaning: 热搜分类 tab，仅支持页面实际提供数据的以下值：realtime(实时热点)、game(游戏)、finance(财经)、sport(体育)、novel(小说)、car(汽车)。其它 tab 返回 400  
**Environment Variables:** no