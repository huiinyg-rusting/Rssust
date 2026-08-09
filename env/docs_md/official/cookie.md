# cookies 配置指南  
因为有些路由需要cookies,而cookies又不好分发，故作此教程以告诉导出哪个网站的cookies,又或者是否需要登录或者额外配置,cookies 示例格式：  
Because some routes require cookies, and cookies are hard to distribute, this tutorial was made to show how to export cookies from a website, or whether login or additional setup is needed. Example cookie format:  
 {  
   "name": "session_id",  
   "value": "abc123",  
   "domain": ".example.com",  
   "path": "/",  
   "secure": true,  
   "httpOnly": true,  
   "sameSite": "Lax",  
   "expirationDate": 1893456000  
 }  

You can use the browser plugin Cookie-Editor to export it, or use the built-in CLI command:

```
rssust cookie <browser>
```

Supported browsers: `firefox`, `chrome` (`chromium`/`chromebeta` 也映射到 chrome)（macOS: `safari`，Windows: `edge`）

Example:
```
./env/rssust cookie chrome
```

导出哪些网址由 `config.toml` 的 `[cookie] origins = ["https://bilibili.com"]` 控制，可自行增删。
已存在于 `cookies.json` 中的域名也会一并刷新。
The URLs to export are controlled by `[cookie] origins = ["https://bilibili.com"]` in `config.toml`; existing domains in `cookies.json` are also refreshed.

Tips: Linux 下 firefox 可能因为上游 crate 的问题需要链接文件夹修复  
## Bilibili  
bilibili.com  
无需登录，访客 cookies 即可  
注意：`bilibili_video_reply` 需要登录后的 cookies（需包含 SESSDATA）
