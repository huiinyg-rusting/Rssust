# Router-name: gelonghui_newsflash  
**Commit time:** 2026.10.08  
**Cookies?:** no  
**Author:** AI转写 / huiinyg审核 - 来源于RSShub  
**Introduction:** 格隆汇实时快讯，支持频道（全部/最热/A股/港股/AI 等）  
**Address:** rssust://gelonghui_newsflash  
**Example:** [rssust://gelonghui_newsflash](/gelonghui_newsflash)  
**Parameter:**  
1. **channel**  
   Type of parameter: string  
   Default value: all  
   Meaning: 快讯频道，取值见下表  

channel 取值：

| 全部 | 最热 | 国际 | AI | 港股 | 虚拟资产 | 中国大陆 |
| ---- | ---- | ---- | -- | ---- | -------- | -------- |
| all | popular | international | ai | HKStock | virtualAssets | chineseMainland |

| A股 | 港澳台 | 美股 | 商品外汇 | 楼市 | 基金 | 债券 |
| --- | ------ | ---- | -------- | ---- | ---- | ---- |
| AStock | hongkongMacaoTaiwan | USStock | exchangeCommodity | house | fundLive | debenture |

> 返回条数由频道决定（多数频道 15 条，最热 popular 为 50 条），接口的 limit 参数服务端会忽略。  
**Environment Variables:** no
