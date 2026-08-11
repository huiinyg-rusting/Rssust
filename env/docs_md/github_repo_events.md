# Router-name: github_repo_events  
**Commit time:** 2026.08.11  
**Cookies?:** no  
**Author:** AI转写 / huiinyg-rusting审核 - 来源于RSShub@mslxl  
**Introduction:** GitHub 仓库事件流 (push/issue/PR/release/star 等)  
**Address:** rssust://github_repo_events  
**Example:** [rssust://github_repo_events?owner=octocat&repo=Hello-World](/github_repo_events?owner=octocat&repo=Hello-World)  
**Parameter:**  
1. **owner**  
   Type of parameter: string  
   Default value: 无  
   Meaning: 仓库所有者 (必填)  
2. **repo**  
   Type of parameter: string  
   Default value: 无  
   Meaning: 仓库名 (必填)  
3. **types**  
   Type of parameter: string  
   Default value: all  
   Meaning: 事件类型过滤, 逗号分隔 (create/delete/fork/issue/issuecomm/member/pr/prcomm/prrev/public/push/release/star/wiki/cmcomm/discussion)  
4. **limit**  
   Type of parameter: number  
   Default value: 30  
   Meaning: 条数, 最大 100
