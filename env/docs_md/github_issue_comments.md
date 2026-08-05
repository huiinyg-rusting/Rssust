# Router-name: github_issue_comments  
**Commit time:** 2026.08.04  
**Cookies?:** no  
**Author:** huiinyg-rusting  
**Introduction:** GitHub repository recent Issue / Pull Request comments via GraphQL API  
**Address:** rssust://github_issue_comments  
**Example:** [rssust://github_issue_comments?owner=facebook&repo=react](/github_issue_comments?owner=facebook&repo=react)  
**Parameter:**  
    owner (required, repository owner)  
    repo (required, repository name)  
    limit (optional, max 50, default 20, split between issues & PRs, 2 latest comments each)  
**Token:**  
    Required. Set the environment variable GITHUB_TOKEN (GitHub Personal Access Token). GitHub recently upgraded anti-crawling; without a token the API is not accessible.