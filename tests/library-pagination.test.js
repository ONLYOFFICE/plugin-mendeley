const assert = require("assert");
const LibraryView = require("../scripts/library-view");

function createElement() {
    return {
        children: [],
        classList: { add() {} },
        appendChild(child) { this.children.push(child); },
        setAttribute() {},
        get lastChild() { return this.children[this.children.length - 1] || null; },
        removeChild(child) { this.children.splice(this.children.indexOf(child), 1); }
    };
}

function waitForPage() {
    return new Promise(resolve => setTimeout(resolve, 80));
}

async function run() {
    const originalWindow = global.window;
    const originalDocument = global.document;
    const holder = createElement();
    const scrollContainer = { scrollTop: 800, clientHeight: 500, scrollHeight: 1500 };
    let requestedPages = 0;

    global.document = { createElement };
    global.window = {
        MendeleyApp: {
            elements: { docsHolder: holder, docsWrapper: scrollContainer },
            Auth: { getCurrentAuthState: () => "main" },
            docsScroller: { onscroll() {} }
        }
    };

    function page() {
        const pageNumber = requestedPages + 1;
        return {
            items: [{ id: "reference-" + pageNumber, title: "Reference " + pageNumber }],
            next() {
                requestedPages += 1;
                return Promise.resolve(page());
            }
        };
    }

    LibraryView.lastSearch.ownObj = page();
    LibraryView.displaySearchItems(true, true, LibraryView.lastSearch.ownObj, null);
    assert.strictEqual(LibraryView.shouldLoadMore(scrollContainer, "main"), true);


    for (let scroll = 0; scroll < 6; scroll += 1) {
        LibraryView.checkDocsScroll(scrollContainer);
        await waitForPage();
    }

    const loadedReferences = holder.children.reduce(function (count, documentPage) {
        return count + documentPage.children.length;
    }, 0);
    assert.strictEqual(requestedPages, 6, "six successive near-bottom scrolls must request six more pages");
    assert.strictEqual(loadedReferences, 7, "initial reference and six following references must render");

    LibraryView.clearLibrary();
    global.window = originalWindow;
    global.document = originalDocument;
    console.log("# Six-page reference loading test passed!");
}

run().catch(error => {
    console.error(error);
    process.exitCode = 1;
});
