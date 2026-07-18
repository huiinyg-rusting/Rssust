# Router-name: cde_index  
**Commit time:** 2026.07.18  
**Cookies?:** no  
**Author:** @Fatpandac  
**Introduction:** 国家药品审评网站首页（新闻中心/政策法规）  
**Address:** rssust://cde_index  
**Example:** [rssust://cde_index?channel=news&category=gzdt](/cde_index?channel=news&category=gzdt)  
**Parameter:**  
1. **channel**  
   Type of parameter: string  
   Default value: null  
   Meaning: 频道  

| channel 值 | 说明     |
| ---------- | -------- |
| news       | 新闻中心 |
| policy     | 政策法规 |
2. **category**  
   Type of parameter: string  
   Default value: null  
   Meaning: 类别  

| channel | category | 说明     |
| ------- | -------- | -------- |
| news    | zwxw     | 政务新闻 |
| news    | ywdd     | 要闻导读 |
| news    | tpxw     | 图片新闻 |
| news    | gzdt     | 工作动态 |
| policy  | flfg     | 法律法规 |
| policy  | zxgz     | 政策规章 |
3. **limit**  
   Type of parameter: number  
   Default value: 25  
   Meaning: 每页条数  
