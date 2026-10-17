# Router-name: wallstreetcn_kuaixun  
**Commit time:** 2026.10.17  
**Cookies?:** no  
**Author:** AI转写 / huiinyg审核 - 来源于RSShub  
**Introduction:** 华尔街见闻 7×24 快讯流，支持频道过滤（公开 JSON API，无官方 RSS）  
**Address:** rssust://wallstreetcn_kuaixun  
**Example:** [rssust://wallstreetcn_kuaixun](/wallstreetcn_kuaixun)  
**Parameter:**  
1. **channel**  
   Type of parameter: string  
   Default value: (空)  
   Meaning: 快讯频道，空=全部 7×24 快讯，取值见下表  
2. **limit**  
   Type of parameter: number  
   Default value: 20  
   Meaning: 返回条数，最大 100（超过 100 上游只返回 100）  

channel 取值：

| 全部 | A股 | 港股 | 美股 | 外汇 | 原油 | 黄金 | 商品 |
| ---- | --- | ---- | ---- | ---- | ---- | ---- | ---- |
| (空) | astock | hkstock | usstock | forex | oil | gold | commodity |

已知的标题多为空字符串（纯直播快讯），此时标题自动回退取正文首句，因此无需担心空标题条目。  
**Environment Variables:** no
