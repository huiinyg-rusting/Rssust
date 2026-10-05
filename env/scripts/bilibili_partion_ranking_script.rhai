//@name B站分区排行榜(脚本示例)
//@source https://api.bilibili.com/x/web-interface/ranking/v2?rid={rid}&type=all

let doc = `# B站分区排行榜(脚本示例)

内置 bilibili_partion_ranking 的脚本版，换用官方 ranking/v2 接口，{rid} 传分区。

- 路由: /bilibili_partion_ranking_script?rid=36
- rid 取值: 0 全部 / 3 音乐 / 4 游戏 / 36 知识 / 188 科技 / 160 生活
`;

let data = parse_json(raw);
let result = [];
for item in data["data"]["list"] {
    let up = item["owner"]["name"];
    result.push(#{
        "title": item["title"],
        "link": "https://www.bilibili.com/video/" + item["bvid"],
        "description": "UP: " + up + " · 播放: " + item["stat"]["view"].to_string(),
        "pub_date": "",
        "author": up
    });
}
return result;