# Router-name: nmc_alarm  
**Commit time:** 2026.07.31  
**Cookies?:** no  
**Author:** AI转写 / huinyg审核 - 自行开发  
**Introduction:** 中央气象台预警信号  
**Address:** rssust://nmc_alarm  
**Example:** [rssust://nmc_alarm?type=暴雨&level=黄色&province=河北](/nmc_alarm?type=%E6%9A%B4%E9%9B%A8&level=%E9%BB%84%E8%89%B2&province=%E6%B2%B3%E5%8C%97)  
**Parameter:**  
1. **type**  
   Type of parameter: string  
   Default value: null  
   Meaning: 预警类型：台风/暴雨/暴雪/寒潮/大风/沙尘暴/高温/干旱/雷电/冰雹/霜冻/大雾/道路结冰  
2. **level**  
   Type of parameter: string  
   Default value: null  
   Meaning: 预警等级：蓝色/黄色/橙色/红色  
3. **province**  
   Type of parameter: string  
   Default value: null  
   Meaning: 省级区域，支持简称（如"河北"）或全称（如"河北省"）  
4. **date**  
   Type of parameter: string  
   Default value: null  
   Meaning: 指定日期（YYYY-MM-DD），只返回该日发布的预警  
5. **limit**  
   Type of parameter: number  
   Default value: 50  
   Meaning: 条目数量上限，最大 200  
