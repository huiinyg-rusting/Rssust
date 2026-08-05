# Router-name: github_followers  
**Commit time:** 2026.08.04  
**Cookies?:** no  
**Author:** huiinyg-rusting  
**Introduction:** GitHub user followers query via GraphQL API  
**Address:** rssust://github_followers  
**Example:** [rssust://github_followers?username=torvalds](/github_followers?username=torvalds)  
**Parameter:**  
    username (required, GitHub login)  
**Token:**  
    Required. Set the environment variable GITHUB_TOKEN (GitHub Personal Access Token). GitHub recently upgraded anti-crawling; without a token the API is not accessible.