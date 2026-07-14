# Router-name: bilibili_collection
**Commit time:** 2026.07.14
**Cookies?:** no
**Author:** AI辅助撰写文档
**Introduction:** B站合集（合集，不是系列）视频列表，通过 season_id 获取合集内的视频
**Address:** rssust://bilibili_collection
**Example:** [rssust://bilibili_collection?uid=245645656&sid=529166](../bilibili_collection?uid=245645656&sid=529166)
**Parameter:**
1. **uid**
   Type of parameter: num
   Default value: null
   Meaning: B站用户 uid（必填）
2. **sid**
   Type of parameter: num
   Default value: null
   Meaning: 合集 season_id（必填）
3. **sortreverse**
   Type of parameter: bool
   Default value: true
   Meaning: 排序方向，true 为倒序，false 为正序
4. **pagenum**
   Type of parameter: num
   Default value: 1
   Meaning: 页码
5. **pagesize**
   Type of parameter: num
   Default value: 16
   Meaning: 每页视频数量
6. **disableembed**
   Type of parameter: bool
   Default value: true
   Meaning: 是否内嵌视频播放器与封面图；设为 false 则仅显示文字描述
**Environment Variables:** no
