// ddoc's Search system, by Canop
// Can be used with most static documentation sites
// Take last version at https://github.com/Canop/ddoc
;window.ddoc_search = (function() {

// [{name,href,body}]
const docs = [];
const tag_score = {
    HTML: 200,
    H1: 100,
    H2: 90,
    H3: 80,
    H4: 70,
    H5: 60,
    H6: 50,
    TABLE: 30,
    UL: 30,
    P: 10,
};
let panel_wrapper = null;
let content_selector = 'main';
let selection_index = -1;

function close() {
    if (panel_wrapper) {
        document.body.removeChild(panel_wrapper);
        panel_wrapper = null;
    }
}

// Add a page to search
async function add_doc(name, href) {
    let already_added = docs.find(doc => doc.href === href);
    if (already_added) {
        // already added (probably because we already searched from this page)
        already_added.name = name; // update name
        return;
    }
    const response = await fetch(href);
    const html = await response.text();
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, 'text/html');
    docs.push({name, href, body: doc.body});
}

// Add docs from all links in menus matching css_selector
async function add_menu_docs(css_selector) {
    const menus = document.querySelectorAll(css_selector);
    for (const menu of menus) {
        const links = menu.querySelectorAll('a');
        for (const link of links) {
            let href = link.getAttribute('href');
            if (/^([^:]*)/.test(href)) {
                const name = link.textContent.trim();
                href = href.replace(/#.*$/, ''); // remove hash
                await add_doc(name, href);
            }
        }
    }
}

// Return [{doc_idx, score, page, section, href, tag, extract}] of matching docs
function search_docs({pattern}) {
    let regex = RegExp(`\\b${pattern}`, 'i');
    let matches = [];
    for (let i = 0; i < docs.length; i++) {
        let doc = docs[i];
        let content = doc.body.querySelectorAll(content_selector);
        let last_hash = '#';
        let last_title = '';
        let page_score = regex.test(doc.name) ? 5 : 0;
        if (page_score > 0) {
            matches.push({
                doc_idx: i,
                page: doc.name,
                href: doc.href,
                score: tag_score.HTML,
            });
        }
        let title_added = false;
        for (let container of content) {
            for (element of container.children) {
                if (element.tagName === 'SCRIPT') {
                    continue;
                }
                let is_title = element.tagName.match(/^H[1-6]$/);
                if (element.id) {
                    last_hash = `#${element.id}`;
                    if (is_title) {
                        last_title = element.textContent.trim();
                    }
                }
                if (!is_title && (title_added || matches.length >= 50)) {
                    continue;
                }
                if (regex.test(element.textContent)) {
                    let score = page_score + tag_score[element.tagName] || 10;
                    let match ={
                        doc_idx: i,
                        page: doc.name,
                        section: last_title,
                        href: `${doc.href}${last_hash}`,
                        tag: element.tagName,
                        score,
                    };
                    title_added = true;
                    if (!is_title && element.tagName !== 'TABLE') {
                        match.extract = element.textContent.trim()
                            .replace(/\s+/g, ' ')
                            .substring(0, 400);
                    }
                    matches.push(match);
                }
            }
        }
    }
    matches.sort((a, b) => b.score - a.score);
    return matches;
}

function highlight_if_needed() {
    const url = new URL(window.location);
    const pattern = url.searchParams.get('search');
    if (pattern) {
        highlight_matches({pattern});
        // we clean the URL to avoid re-highlighting on reload
        url.searchParams.delete('search');
        window.history.replaceState({}, document.title, url.toString());
    }
}

// wrap matches with <mark> in the current page
function highlight_matches({pattern}) {
    let regex = RegExp(`\\b${pattern}`, 'ig');
    let content = document.body.querySelectorAll(content_selector);
    for (let container of content) {
        highlight_matches_in_element(regex, container);
    }
}
function highlight_matches_in_element(regex, element) {
    // Walk through all text nodes
    const walker = document.createTreeWalker(
        element,
        NodeFilter.SHOW_TEXT,
        {
            acceptNode: function(node) {
                const parent = node.parentElement;
                if (parent.tagName === 'SCRIPT' ||
                    parent.tagName === 'STYLE' ||
                    parent.tagName === 'HEADER' ||
                    parent.tagName === 'FOOTER' ||
                    parent.tagName === 'MARK'
                ) {
                    return NodeFilter.FILTER_REJECT;
                }
                return NodeFilter.FILTER_ACCEPT;
            }
        }
    );

    const textNodes = [];
    let currentNode;

    // Collect all text nodes first (to avoid modifying during traversal)
    while (currentNode = walker.nextNode()) {
        textNodes.push(currentNode);
    }

    // Process each text node
    textNodes.forEach(node => {
        const text = node.textContent;
        const matches = [...text.matchAll(regex)];
        if (matches.length === 0) return;

        const fragment = document.createDocumentFragment();
        let lastIndex = 0;

        matches.forEach(match => {
            const matchStart = match.index;
            const matchEnd = matchStart + match[0].length;

            // Add text before match
            if (matchStart > lastIndex) {
                fragment.appendChild(
                    document.createTextNode(text.slice(lastIndex, matchStart))
                );
            }

            // Add highlighted match
            const mark = document.createElement('mark');
            mark.textContent = match[0];
            fragment.appendChild(mark);

            lastIndex = matchEnd;
        });

        // Add remaining text after last match
        if (lastIndex < text.length) {
            fragment.appendChild(
                document.createTextNode(text.slice(lastIndex))
            );
        }

        // Replace original text node with fragment
        node.parentNode.replaceChild(fragment, node);
    });
}

function open_search_panel() {
    let wrapper = document.createElement('div');
    wrapper.className = 'ddoc-search-panel-wrapper';
    wrapper.addEventListener('click', close);
    let panel = document.createElement('div');
    panel.className = 'ddoc-search-panel';
    panel.addEventListener('click', function(event) {
        event.stopPropagation();
        return false;
    });
    wrapper.appendChild(panel);
    let controls = document.createElement('div');
    controls.className = 'ddoc-search-controls';
    panel.appendChild(controls);
    let input = document.createElement('input');
    input.type = 'text';
    controls.appendChild(input);
    let closer = document.createElement('a');
    closer.className = 'ddoc-search-close';
    closer.addEventListener('click', close);
    closer.textContent = 'X';
    controls.appendChild(closer);
    let results = document.createElement('div');
    results.className = 'ddoc-search-results';
    panel.appendChild(results);
    document.body.appendChild(wrapper);
    panel_wrapper = wrapper;
    input.focus();
    // update search results on input
    input.addEventListener('input', function(event) {
        results.innerHTML = '';
        selection_index = -1;
        let pattern = input.value.trim();
        if (pattern.length === 0) {
            return;
        }
        let matches = search_docs({
            pattern,
        });
        if (matches.length === 0) {
            results.innerHTML = '<span class=ddoc-search-no-result>No results</span>';
            return;
        }
        for (let match of matches) {
            let item = document.createElement('div');
            item.className = 'ddoc-search-result';
            let path = document.createElement('div');
            path.className = 'ddoc-search-result-path';
            let page_link = document.createElement('a');
            page_link.href = match.href.split('#')[0];
            page_link.href = append_param(page_link.href, 'search', pattern);
            path.addEventListener('click', close);
            page_link.textContent = match.page;
            path.appendChild(page_link);
            if (match.section) {
                let sep = document.createElement('span');
                sep.textContent = ' > ';
                sep.className = 'ddoc-search-result-sep';
                path.appendChild(sep);
                let section_link = document.createElement('a');
                section_link.href = append_param(match.href, 'search', pattern);
                section_link.textContent = match.section;
                path.appendChild(section_link);
            }
            item.appendChild(path);
            if (match.extract) {
                let extract = document.createElement('div');
                extract.className = 'ddoc-search-result-extract';
                extract.textContent = match.extract;
                item.appendChild(extract);
            }
            results.appendChild(item);
        }
    });
    // handle keyboard navigation: up/down + enter
    input.addEventListener('keydown', function(event) {
        let items = results.querySelectorAll('.ddoc-search-result');
        if (event.key === 'ArrowDown') {
            // down
            if (items.length === 0) return;
            selection_index++;
            if (selection_index >= items.length) {
                selection_index = 0;
            }
            items.forEach((item, idx) => {
                if (idx === selection_index) {
                    item.classList.add('ddoc-search-result-selected');
                    item.scrollIntoView({block: 'nearest'});
                } else {
                    item.classList.remove('ddoc-search-result-selected');
                }
            });
            event.preventDefault();
        } else if (event.key === 'ArrowUp') {
            // up
            if (items.length === 0) return;
            selection_index--;
            if (selection_index < 0) {
                selection_index = items.length - 1;
            }
            items.forEach((item, idx) => {
                if (idx === selection_index) {
                    item.classList.add('ddoc-search-result-selected');
                    item.scrollIntoView({block: 'nearest'});
                } else {
                    item.classList.remove('ddoc-search-result-selected');
                }
            });
            event.preventDefault();
        } else if (event.key === 'Enter') {
            // enter
            if (selection_index >= 0 && selection_index < items.length) {
                let links = items[selection_index].querySelectorAll('a');
                if (links.length > 0) {
                    // when building results, we may set up more than one link
                    // (eg: page and section), we take the last one which
                    // is the most precise
                    window.location.href = links[links.length-1].href;
                }
            }
        }
    });
}

// Append or set a query parameter to a URL (which may be relative and contain '..')
function append_param(href, key, value) {
    let parts = href.split('#');
    if (parts[0].includes('?')) {
        parts[0] += '&';
    } else {
        parts[0] += '?';
    }
    parts[0] += `${key}=${encodeURIComponent(value)}`;
    return parts.join('#');
}

async function open(options = {}) {
    open_search_panel();
    prepare(options);
}

// options:{
//  menu_selector: css selector of menu element(s) to pull docs from
//  content_selector: css selector of element parent of content to search
// }
async function prepare(options = {}) {
    if (options.menu_selector) {
        await add_menu_docs(options.menu_selector);
    }
    if (options.content_selector) {
        content_selector = options.content_selector;
    }
}

// close search panel on escape key
document.addEventListener('keyup', function(event) {
    if (event.key === 'Escape') {
        close();
    }
});

return {
    open,
    prepare,
    add_menu_docs,
    close,
    highlight_matches,
    highlight_if_needed,
};

})();

window.addEventListener("load", (event) => {
    ddoc_search.prepare({
        menu_selector: ".nav-menu",
        content_selector: "main",
    });
    ddoc_search.highlight_if_needed();
});
