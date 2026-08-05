# Router-name: github_commits  
**Commit time:** 2026.08.04  
**Cookies?:** no  
**Author:** huiinyg-rusting  
**Introduction:** GitHub repository recent commits on default branch via GraphQL API  
**Address:** rssust://github_commits  
**Example:** [rssust://github_commits?owner=rust-lang&repo=rust](/github_commits?owner=rust-lang&repo=rust)  
**Parameter:**  
    owner (required, repository owner)  
    repo (required, repository name)  
    limit (optional, max 50, default 10)  
**Token:**  
    Required. Set the environment variable GITHUB_TOKEN (GitHub Personal Access Token). GitHub recently upgraded anti-crawling; without a token the API is not accessible.