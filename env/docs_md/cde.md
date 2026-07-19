# Router-name: cde  
**Commit time:** 2026.07.18  
**Cookies?:** no  
**Author:** AI转写 / huinyg审核 - 来源于RSShub@Fatpandac  
**Introduction:** 国家药品审评网站（新闻中心/政策法规）  
**Address:** rssust://cde  
**Example:** [rssust://cde?channel=news&category=gzdt](/cde?channel=news&category=gzdt)  
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

**Note:**  
- 新闻频道详情页内容完整（包含 full HTML 描述）  
- 政策频道详情页内容通过 JavaScript 动态渲染，HTTP 客户端无法获取，描述字段为空  
