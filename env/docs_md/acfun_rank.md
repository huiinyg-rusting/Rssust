# Router-name: acfun_rank  
**Commit time:** 2026.10.23  
**Cookies?:** no  
**Author:** AI转写 / huiinyg审核 - 参考RSShub  
**Introduction:** AcFun 排行榜（日榜/三日榜/周榜），数据取自 AcFun 站内排行榜接口，无官方 RSS  
**Address:** rssust://acfun_rank  
**Example:** [rssust://acfun_rank](/acfun_rank)  
**robots.txt:** www.acfun.cn 无 robots.txt，视为允许  
**Parameter:**  
1. **period**  
   Type of parameter: string  
   Default value: day  
   Meaning: 榜单周期，支持 day(日榜)、three_days(三日榜)、week(周榜)；上游不支持月榜（MONTH 返回 result=21）  
2. **limit**  
   Type of parameter: number  
   Default value: 30  
   Meaning: 返回条数，范围 1-100，超出会被钳制  

示例：周榜 50 条 `rssust://acfun_rank?period=week&limit=50`

> 榜单为全站排行，接口本身不分页；单次最多取 100 条。
**Environment Variables:** no
