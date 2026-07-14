# Router-name: bilibili_dynamic
**Commit time:** 2026.7.11
**Cookies?:** yes (使用`cookies.json`直接注入到reqwest,需要鲜活的cookies,反爬中等)
**Author:** AI大部分转写自RSShub/huiinyg-rusting收拾烂摊子和审核-AGPL-3.0协议（已被传染）
**Introduction:** 获取指定 Bilibili 用户的动态（综合、视频、图文、直播、转发等），生成 RSS 源。入口函数 `pub fn get(para: HashMap<String, String>) -> Result<String, Error>`，位于 `src/router/bilibili_dynamic.rs:405`。内部调用 Bilibili 官方 API `/x/polymer/web-dynamic/v1/feed/space`。

**Address:** `rssust://bilibili_dynamic`
**Example:** [/bilibili_dynamic?uid=538142100&hideGoods=true](../bilibili_dynamic?uid=538142100&hideGoods=true)
**Parameter:**
1. **uid**
   - Type of parameter: string (必填)
   - Default value: 无
   - Meaning: Bilibili 用户 UID
2. **showEmoji**
   - Type of parameter: bool
   - Default value: false
   - Meaning: 是否将动态正文中的 emoji 文本替换为图片
3. **embed**
   - Type of parameter: bool
   - Default value: true
   - Meaning: 视频动态是否内嵌 iframe 播放器
4. **useAvid**
   - Type of parameter: bool
   - Default value: false
   - Meaning: 视频链接使用 av 号（默认使用 bv 号）
5. **directLink**
   - Type of parameter: bool
   - Default value: false
   - Meaning: 链接直接跳转至内容页而非动态页
6. **hideGoods**
   - Type of parameter: bool
   - Default value: false
   - Meaning: 是否隐藏带货类动态
7. **offset**
   - Type of parameter: string
   - Default value: 空
   - Meaning: 翻页偏移量（从上次返回的 `data.offset` 取值，用于分页续抓）
  
**Environment Variables:** no
