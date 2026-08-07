# Router-name: github_advisor  
**Commit time:** 2026.08.06  
**Cookies?:** no  
**Author:** huiinyg-rusting  
**Introduction:** GitHub Advisory Database Security Advisories (REST API)  
**Address:** rssust://github_advisor  
**Example:** [rssust://github_advisor?type=reviewed&ecosystem=npm](/github_advisor?type=reviewed&ecosystem=npm)  
**Parameter:**  
    type (optional, reviewed=reviewed unreviewed=unreviewed, default reviewed)  
    ecosystem (optional, composer/go/maven/npm/nuget/pip/pub/rubygems/rust/erlang/actions/swift, default all)  
    limit (optional, max 50, default 20)  
**Token:**  
    Required. Set the environment variable GITHUB_TOKEN (GitHub Personal Access Token). GitHub recently upgraded anti-crawling; without a token the API is not accessible.