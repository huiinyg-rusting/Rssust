# 环境变量配置指南  
因为有些路由需要配置环境变量才能正常工作，例如 GitHub 加强了反爬，`github_*` 路由不带令牌时 API 不可访问，故作此说明以告知哪些路由需要哪个环境变量，以及如何设置。  
Because some routes need environment variables to work properly — for example, GitHub has strengthened anti-crawling, so the `github_*` routes cannot reach the API without a token — this page explains which routes need which environment variables and how to set them.  

设置环境变量（Linux/macOS 临时生效）：  
```
export GITHUB_TOKEN=ghp_xxx
```
永久生效可写入 `~/.bashrc` 或 `~/.zshrc`。  

设置环境变量（Windows）：  
```
setx GITHUB_TOKEN ghp_xxx
```
PowerShell 临时生效：  
```
$env:GITHUB_TOKEN = "ghp_xxx"
```

环境变量由启动服务器的进程读取，设置后需先使其生效，再启动服务。  
The variables are read by the process that starts the server; set them first, then start the service.  

## GITHUB_TOKEN  
GitHub 访问令牌（Personal Access Token），供 `github_*` 系列路由调用 GitHub API 使用。  
- 令牌在 GitHub → Settings → Developer settings → Personal access tokens 生成（classic 或 fine-grained），只显示一次，请保存。  
- 多数 `github_*` 路由只读公开数据，classic 令牌无需额外权限；访问私有仓库或 GraphQL 接口需按需授权（如 `repo`、`read:user`）。  
- 未配置时对应路由返回 500，并提示 `Environment variable GITHUB_TOKEN is required`。