# Router-name: openai_chatgpt_atlas_release  
**Commit time:** 2026.08.06  
**Cookies?:** no  
**平台限制:** 抓取 help.openai.com 需 `curl-impersonate`（chrome110 TLS 指纹）。Windows x86_64 无预编译二进制 → 需从源码编译 curl-impersonate 或使用 WSL；否则本路由 500（无降级）。参考 `openai_common.rs::fetch_release_notes`  
**Author:** AI-converted / huiinyg-reviewed - based on RSSHub  
**Introduction:** ChatGPT Atlas Release Notes (help.openai.com single page scrape)  
**Address:** rssust://openai_chatgpt_atlas_release  
**Example:** [rssust://openai_chatgpt_atlas_release](/openai_chatgpt_atlas_release)  
**Parameter:**  
    limit (optional, max 30, default 10)