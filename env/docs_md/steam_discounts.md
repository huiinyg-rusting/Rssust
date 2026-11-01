# Router-name: steam_discounts
**Commit time:** 2026.10.31
**Cookies?:** no
**Author:** AI转写 / huiinyg审核 - 来源于RSShub@DIYgod  
**Introduction:** Steam 首页“特惠”区的打折游戏，含折扣率、原价/现价与折扣截止时间；数据来自 Steam 官方公开 JSON 
**Address:** rssust://steam_discounts
**Example:** [rssust://steam_discounts](/steam_discounts)
**robots.txt:** store.steampowered.com 的 robots.txt 只禁止账号/礼物等路径，未禁止 /api/；建议限流 600s
**Parameter:**
1. **cc**
   Type of parameter: string
   Default value: cn
   Meaning: 国家/地区代码，价格与货币随地区变化（如 us / jp / hk）
2. **lang**
   Type of parameter: string
   Default value: zh
   Meaning: 语言代码（页面文字，不影响价格）
3. **limit**
   Type of parameter: number
   Default value: 10
   Meaning: 返回条数，最大 20（上游特惠区一次约 10 条）
**Environment Variables:** no
