# Router-name: eastmoney_kuaixun  
**Commit time:** 2026.10.17  
**Cookies?:** no  
**Author:** AI转写 / huiinyg审核 - 来源于RSShub  
**Introduction:** 东方财富 7×24 全球财经快讯，支持栏目过滤与翻页（公开 JSON API，无官方 RSS）  
**Address:** rssust://eastmoney_kuaixun  
**Example:** [rssust://eastmoney_kuaixun](/eastmoney_kuaixun)  
**Parameter:**  
1. **column**  
   Type of parameter: string  
   Default value: 102  
   Meaning: 栏目 ID，空=102 全球财经，取值见下表  
2. **limit**  
   Type of parameter: number  
   Default value: 20  
   Meaning: 返回条数，最大 200（超过 200 上游只返回 200）  
3. **sort_end**  
   Type of parameter: string  
   Default value: (空)  
   Meaning: 翻页游标，取上一页最后一条对应的 realSort 值（形如 1792239360072889 的整数串）以加载更早的快讯  

column 取值：

| ID | 栏目 | ID | 栏目 |
| -- | ---- | -- | ---- |
| 101 | 焦点 | 102 | 全球财经 |
| 103 | 上市公司 | 104 | A股 |
| 107 | 外汇 | 108 | 债券 |
| 109 | 基金 | 110 | 地区-中国 |
| 111 | 地区-美国 | 112 | 地区-欧元区 |
| 113 | 地区-英国 | 114 | 地区-日本 |
| 115 | 地区-加拿大 | 116 | 地区-澳洲 |
| 117 | 地区-新兴市场 | 118 | 央行-中国央行 |
| 119 | 央行-美联储 | 120 | 央行-欧洲央行 |
| 121 | 央行-英国央行 | 122 | 央行-日本央行 |
| 123 | 央行-加拿大央行 | 124 | 央行-澳洲联储 |
| 125 | 数据-中国 | 126 | 数据-美国 |
| 127 | 数据-欧元区 | 128 | 数据-英国 |
| 129 | 数据-日本 | 130 | 数据-加拿大 |
| 131 | 数据-澳洲 |  |  |

示例：焦点 `rssust://eastmoney_kuaixun?column=101`；翻页 `rssust://eastmoney_kuaixun?sort_end=1792239360072889`  
**Environment Variables:** no
