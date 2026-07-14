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
You can use the browser plugin Cookie-Editor to export it.

Tips:现在自动cookies爬取已经出了，只要加` 二进制文件 cookies 浏览器名`
但是实测linux下的firefox出了一些问题，上游crate的问题，可以通过链接文件夹修复
## Bilibili
bilibili.com
无需登录