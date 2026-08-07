// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="api.html">api</a></li><li class="chapter-item expanded "><a href="config.html">config</a></li><li class="chapter-item expanded "><a href="cookie.html">cookie</a></li><li class="chapter-item expanded "><a href="easyuser_cn.html">easyuser_cn</a></li><li class="chapter-item expanded "><a href="guide_cn.html">guide_cn</a></li><li class="chapter-item expanded "><a href="guide_en.html">guide_en</a></li><li class="chapter-item expanded "><a href="new_router_cn.html">new_router_cn</a></li><li class="chapter-item expanded "><a href="new_router_en.html">new_router_en</a></li><li class="chapter-item expanded "><a href="pogram_explanation_cn.html">pogram_explanation_cn</a></li><li class="chapter-item expanded affix "><li class="part-title">路由</li><li class="chapter-item expanded "><a href="apnews_topics.html">apnews_topics</a></li><li class="chapter-item expanded "><a href="bilibili_collection.html">bilibili_collection</a></li><li class="chapter-item expanded "><a href="bilibili_dynamic.html">bilibili_dynamic</a></li><li class="chapter-item expanded "><a href="bilibili_fav.html">bilibili_fav</a></li><li class="chapter-item expanded "><a href="bilibili_link_news.html">bilibili_link_news</a></li><li class="chapter-item expanded "><a href="bilibili_partion.html">bilibili_partion</a></li><li class="chapter-item expanded "><a href="bilibili_partion_ranking.html">bilibili_partion_ranking</a></li><li class="chapter-item expanded "><a href="bilibili_popular.html">bilibili_popular</a></li><li class="chapter-item expanded "><a href="bilibili_precious.html">bilibili_precious</a></li><li class="chapter-item expanded "><a href="bilibili_series.html">bilibili_series</a></li><li class="chapter-item expanded "><a href="bilibili_user_article.html">bilibili_user_article</a></li><li class="chapter-item expanded "><a href="bilibili_user_coin.html">bilibili_user_coin</a></li><li class="chapter-item expanded "><a href="bilibili_user_fav.html">bilibili_user_fav</a></li><li class="chapter-item expanded "><a href="bilibili_user_like.html">bilibili_user_like</a></li><li class="chapter-item expanded "><a href="bilibili_video_page.html">bilibili_video_page</a></li><li class="chapter-item expanded "><a href="bilibili_video_reply.html">bilibili_video_reply</a></li><li class="chapter-item expanded "><a href="bilibili_vsearch.html">bilibili_vsearch</a></li><li class="chapter-item expanded "><a href="bilibili_weekly.html">bilibili_weekly</a></li><li class="chapter-item expanded "><a href="bjnews_cat.html">bjnews_cat</a></li><li class="chapter-item expanded "><a href="caixin_latest.html">caixin_latest</a></li><li class="chapter-item expanded "><a href="carnegieendowment_news.html">carnegieendowment_news</a></li><li class="chapter-item expanded "><a href="cenc_earthquake.html">cenc_earthquake</a></li><li class="chapter-item expanded "><a href="chinanews.html">chinanews</a></li><li class="chapter-item expanded "><a href="cls_hot.html">cls_hot</a></li><li class="chapter-item expanded "><a href="defensenews_news.html">defensenews_news</a></li><li class="chapter-item expanded "><a href="defenseone_news.html">defenseone_news</a></li><li class="chapter-item expanded "><a href="devto_guides.html">devto_guides</a></li><li class="chapter-item expanded "><a href="discovermagazine_news.html">discovermagazine_news</a></li><li class="chapter-item expanded "><a href="douban_book_latest.html">douban_book_latest</a></li><li class="chapter-item expanded "><a href="douban_book_rank.html">douban_book_rank</a></li><li class="chapter-item expanded "><a href="douban_event_hot.html">douban_event_hot</a></li><li class="chapter-item expanded "><a href="douban_movie_classification.html">douban_movie_classification</a></li><li class="chapter-item expanded "><a href="eastday_24.html">eastday_24</a></li><li class="chapter-item expanded "><a href="eeo_kuaixun.html">eeo_kuaixun</a></li><li class="chapter-item expanded "><a href="gelonghui_home.html">gelonghui_home</a></li><li class="chapter-item expanded "><a href="github_advisor.html">github_advisor</a></li><li class="chapter-item expanded "><a href="github_commits.html">github_commits</a></li><li class="chapter-item expanded "><a href="github_followers.html">github_followers</a></li><li class="chapter-item expanded "><a href="github_issue_comments.html">github_issue_comments</a></li><li class="chapter-item expanded "><a href="github_stars.html">github_stars</a></li><li class="chapter-item expanded "><a href="guancha_headline.html">guancha_headline</a></li><li class="chapter-item expanded "><a href="guanhai.html">guanhai</a></li><li class="chapter-item expanded "><a href="guokr_scientific.html">guokr_scientific</a></li><li class="chapter-item expanded "><a href="hackernews.html">hackernews</a></li><li class="chapter-item expanded "><a href="ifeng_news.html">ifeng_news</a></li><li class="chapter-item expanded "><a href="ithome_ranking.html">ithome_ranking</a></li><li class="chapter-item expanded "><a href="jianshu_home.html">jianshu_home</a></li><li class="chapter-item expanded "><a href="juejin_pins.html">juejin_pins</a></li><li class="chapter-item expanded "><a href="juejin_trending.html">juejin_trending</a></li><li class="chapter-item expanded "><a href="leiphone_newsflash.html">leiphone_newsflash</a></li><li class="chapter-item expanded "><a href="mittrchina.html">mittrchina</a></li><li class="chapter-item expanded "><a href="netease_today.html">netease_today</a></li><li class="chapter-item expanded "><a href="nmc_alarm.html">nmc_alarm</a></li><li class="chapter-item expanded "><a href="openai_chatgpt_atlas_release.html">openai_chatgpt_atlas_release</a></li><li class="chapter-item expanded "><a href="openai_chatgpt_release.html">openai_chatgpt_release</a></li><li class="chapter-item expanded "><a href="openai_news.html">openai_news</a></li><li class="chapter-item expanded "><a href="openai_research.html">openai_research</a></li><li class="chapter-item expanded "><a href="rail12306_news.html">rail12306_news</a></li><li class="chapter-item expanded "><a href="rail12306_ticket.html">rail12306_ticket</a></li><li class="chapter-item expanded "><a href="scientificamerican_news.html">scientificamerican_news</a></li><li class="chapter-item expanded "><a href="smithsonianmag_news.html">smithsonianmag_news</a></li><li class="chapter-item expanded "><a href="solidot.html">solidot</a></li><li class="chapter-item expanded "><a href="stcn_article_list.html">stcn_article_list</a></li><li class="chapter-item expanded "><a href="stcn_kx.html">stcn_kx</a></li><li class="chapter-item expanded "><a href="stcn_rank.html">stcn_rank</a></li><li class="chapter-item expanded "><a href="thepaper_featured.html">thepaper_featured</a></li><li class="chapter-item expanded "><a href="tmtpost_new.html">tmtpost_new</a></li><li class="chapter-item expanded "><a href="videocardz_news.html">videocardz_news</a></li><li class="chapter-item expanded "><a href="wallstreetcn_hot.html">wallstreetcn_hot</a></li><li class="chapter-item expanded "><a href="yicai_headline.html">yicai_headline</a></li><li class="chapter-item expanded "><a href="yicai_latest.html">yicai_latest</a></li><li class="chapter-item expanded "><a href="zhihu_hot.html">zhihu_hot</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
