# Test Multi-Page Reference Loading

## Summary
Added a `LibraryView.checkDocsScroll` regression test that requests six additional pages and verifies all seven returned references render in the library list.

## Verification
- `node tests/library-pagination.test.js`
- `node tests/scroll.test.js`
- `node tests/modules.test.js`
