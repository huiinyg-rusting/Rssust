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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="new_router_en.html"><strong aria-hidden="true">1.</strong> new_router_en</a></li><li class="chapter-item expanded "><a href="config.html"><strong aria-hidden="true">2.</strong> config</a></li><li class="chapter-item expanded "><a href="guide_cn.html"><strong aria-hidden="true">3.</strong> guide_cn</a></li><li class="chapter-item expanded "><a href="api.html"><strong aria-hidden="true">4.</strong> api</a></li><li class="chapter-item expanded "><a href="new_router_cn.html"><strong aria-hidden="true">5.</strong> new_router_cn</a></li><li class="chapter-item expanded "><a href="easyuser_cn.html"><strong aria-hidden="true">6.</strong> easyuser_cn</a></li><li class="chapter-item expanded "><a href="cookie.html"><strong aria-hidden="true">7.</strong> cookie</a></li><li class="chapter-item expanded "><a href="guide_en.html"><strong aria-hidden="true">8.</strong> guide_en</a></li><li class="chapter-item expanded "><a href="pogram_explanation_cn.html"><strong aria-hidden="true">9.</strong> pogram_explanation_cn</a></li><li class="chapter-item expanded affix "><li class="part-title">路由</li><li class="chapter-item expanded "><a href="bilibili_dynamic.html"><strong aria-hidden="true">10.</strong> bilibili_dynamic</a></li><li class="chapter-item expanded "><a href="bilibili_precious.html"><strong aria-hidden="true">11.</strong> bilibili_precious</a></li><li class="chapter-item expanded "><a href="bilibili_collection.html"><strong aria-hidden="true">12.</strong> bilibili_collection</a></li><li class="chapter-item expanded "><a href="bilibili_popular.html"><strong aria-hidden="true">13.</strong> bilibili_popular</a></li><li class="chapter-item expanded "><a href="bilibili_weekly.html"><strong aria-hidden="true">14.</strong> bilibili_weekly</a></li><li class="chapter-item expanded "><a href="bilibili_series.html"><strong aria-hidden="true">15.</strong> bilibili_series</a></li></ol>';
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
