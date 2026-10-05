//@name B站热搜(脚本示例)
//@source https://s.search.bilibili.com/main/hotword

let doc = `# B站热搜(脚本示例)

内置 bilibili_search_hot 的脚本版：Rust 侧抓取后注入 raw，脚本只做 JSON 解析。

- 路由: /bilibili_search_hot_script
- 上游: https://s.search.bilibili.com/main/hotword
`;

let data = parse_json(raw);
let result = [];
if data["code"] == 0 {
    for item in data["list"] {
        result.push(#{
            "title": "#" + item["pos"].to_string() + " " + item["show_name"],
            "link": "https://search.bilibili.com/all?keyword=" + item["keyword"],
            "description": "热搜词: " + item["show_name"],
            "pub_date": ""
        });
    }
}
return result;