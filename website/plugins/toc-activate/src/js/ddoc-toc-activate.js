// ddoc script to highlight the visible section in the table of contents, by Canop
;window.addEventListener('DOMContentLoaded', () => {

    const tocEntries = Array.from(document.querySelectorAll('.toc-item'))
        .filter(e => e.offsetParent !== null); // only visible ones
    const activableIds = tocEntries.map(entry => {
        const link = entry.querySelector('a');
        const href = link.getAttribute('href');
        return href.startsWith('#') ? href.substring(1) : null;
    }).filter(id => id !== null);
    const headings = Array.from(document.querySelectorAll('h1[id],h2[id],h3[id],h4[id]'))
        .filter(h => activableIds.includes(h.id));

    let activeId = null;
    let detection_disabled = false;

    // Trigger recomputation on position change
    const observer = new IntersectionObserver(detectActiveHeading);
    document.querySelectorAll('h1,h2,h3,h4,p,pre,table,img').forEach(e => observer.observe(e));

    function detectActiveHeading() {
        if (detection_disabled) return;
        for (const heading of headings) {
            const rect = heading.getBoundingClientRect();
            if (rect.top > 20 && rect.bottom < window.innerHeight) {
                activeId = heading.id;
                break;
            }
            if (rect.bottom > window.innerHeight) {
                break;
            }
            activeId = heading.id;
        }
        updateActiveTocLink();
    }

    function updateActiveTocLink() {
        tocEntries.forEach(entry => {
            const link = entry.querySelector('a');
            const href = link.getAttribute('href');
            entry.classList.toggle(
                'active',
                href === `#${activeId}`
            );
        });
    }

    document.querySelectorAll('.toc-item a').forEach(link => {
        link.addEventListener('click', () => {
            let id = link.getAttribute('href').substring(1);
            if (id) activeId = id;
            // Force recompute after browser scrolls, and disable
            // detection for some time so that the intersection observer
            // doesn't override our manual selection.
            detection_disabled = true;
            requestAnimationFrame(updateActiveTocLink);
            setTimeout(() => { detection_disabled = false;  }, 10);
        });
    });

    // Ensure we can advance to the very start or very end by scrolling
    window.addEventListener('wheel',
        (e) => {
            if (e.deltaY < 0) { // up scroll
                const atTop = window.scrollY <= 1;
                if (atTop) {
                    let idx = headings.findIndex(h => h.id === activeId);
                    if (idx > 0) {
                        activeId = headings[idx - 1].id;
                        updateActiveTocLink();
                    }
                }
            } else if (e.deltaY > 0) { // down scroll
                const viewportHeight = window.innerHeight;
                const docHeight = document.documentElement.scrollHeight;
                const atBottom = window.scrollY + viewportHeight >= docHeight - 1;
                if (atBottom) {
                    let idx = headings.findIndex(h => h.id === activeId);
                    if (idx >= 0 && idx < headings.length - 1) {
                        activeId = headings[idx + 1].id;
                        updateActiveTocLink();
                    }
                }
            }
        },
        { passive: true }
    );

    detectActiveHeading();
});
