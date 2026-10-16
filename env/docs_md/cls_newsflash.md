# Router-name: cls_newsflash  
**Commit time:** 2026.10.08  
**Cookies?:** no  
**Author:** AI转写 / huiinyg审核 - 来源于RSShub  
**Introduction:** 财联社实时电报快讯，支持频道过滤（全部/看盘/公告/基金要闻/港股/精选）  
**Address:** rssust://cls_newsflash  
**Example:** [rssust://cls_newsflash](/cls_newsflash)  
**Parameter:**  
1. **category**  
   Type of parameter: string  
   Default value: (空)  
   Meaning: 频道过滤，空=全部快讯，取值见下表  
2. **rn**  
   Type of parameter: number  
   Default value: 20  
   Meaning: 返回条数，最大 50（超过 50 上游直接返回空）  
3. **last_time**  
   Type of parameter: number  
   Default value: (当前时间戳)  
   Meaning: 翻页游标，传上一页最后一条的 ctime（Unix 秒）取更早的快讯  

category 取值：

| 全部 | 看盘 | 公告 | 基金要闻 | 港股 | 精选 |
| ---- | ---- | ---- | -------- | ---- | ---- |
| (空) | watch | announcement | jjyw | hk | red |

示例：看盘频道 `rssust://cls_newsflash?category=watch`；翻到更早 `rssust://cls_newsflash?last_time=1700000000`

**Environment Variables:** no
