# Router-name: bilibili_live_area  
**Commit time:** 2026.08.11  
**Cookies?:** no  
**Author:** AI转写 / huiinyg-rusting审核 - 来源于RSShub@Qixingchen  
**Introduction:** B站直播分区 (某分区下房间列表)  
**Address:** rssust://bilibili_live_area  
**Example:** [rssust://bilibili_live_area?area_id=1&order=online](/bilibili_live_area?area_id=1&order=online)  
**Parameter:**  
1. **area_id**  
   Type of parameter: string / number  
   Default value: 无  
   Meaning: 直播分区ID。支持父分区 id（如 1=娱乐，自动展开其下全部子分区合并房间）或子分区 id（如 86=英雄联盟）。可通过 room/v1/Area/getList 查询，新版接口只接受子分区 id 查房间。父分区子分区超过 20 个时报错，请改用子分区 id  
2. **order**  
   Type of parameter: string  
   Default value: 无  
   Meaning: 排序方式, live_time=最新开播 / online=人气直播  
3. **page_size**  
   Type of parameter: number  
   Default value: 30  
   Meaning: 每页条数, 最大 30  
4. **page_no**  
   Type of parameter: number  
   Default value: 1  
   Meaning: 页码
**Environment Variables:** no
