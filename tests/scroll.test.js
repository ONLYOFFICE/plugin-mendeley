const assert = require("assert");

// Unit tests for shouldLoadMore logic
function shouldLoadMore(container, authState, waitForLoad, hasNextPage, threshold = 250) {
    if (authState !== "main") return false;
    if (waitForLoad) return false;
    if (!hasNextPage) return false;

    const currentScroll = container.scrollTop + container.clientHeight;
    const maxScroll = container.scrollHeight;
    return currentScroll >= (maxScroll - threshold);
}

function runScrollTests() {
    console.log("# Testing Infinite Scroll shouldLoadMore Logic");

    const scrollContainer = {
        scrollTop: 0,
        clientHeight: 500,
        scrollHeight: 2000
    };

    // 1. Initial position (at top) -> should not load
    assert.strictEqual(shouldLoadMore(scrollContainer, "main", false, true), false);

    // 2. Scrolled midway (scrollTop = 800) -> should not load
    scrollContainer.scrollTop = 800;
    assert.strictEqual(shouldLoadMore(scrollContainer, "main", false, true), false);

    // 3. Scrolled near bottom (within 250px: scrollTop = 1300, total = 1800, max = 2000) -> should load!
    scrollContainer.scrollTop = 1300;
    assert.strictEqual(shouldLoadMore(scrollContainer, "main", false, true), true);

    // 4. Scrolled near bottom but already waiting for load -> should not load (prevents duplicate requests)
    assert.strictEqual(shouldLoadMore(scrollContainer, "main", true, true), false);

    // 5. Scrolled near bottom but no next page exists -> should not load
    assert.strictEqual(shouldLoadMore(scrollContainer, "main", false, false), false);

    // 6. Not in main state -> should not load
    assert.strictEqual(shouldLoadMore(scrollContainer, "login", false, true), false);

    console.log("# Infinite scroll tests passed!");
}

runScrollTests();
