# Router-name: github_stars  
**Commit time:** 2026.08.04  
**Cookies?:** no  
**Author:** huiinyg-rusting  
**Introduction:** GitHub single repository star count via GraphQL API  
**Address:** rssust://github_stars  
**Example:** [rssust://github_stars?owner=torvalds&repo=linux](/github_stars?owner=torvalds&repo=linux)  
**Parameter:**  
    owner (required, repository owner)  
    repo (required, repository name)  
**Token:**  
    Required. Set the environment variable GITHUB_TOKEN (GitHub Personal Access Token). GitHub recently upgraded anti-crawling; without a token the API is not accessible.