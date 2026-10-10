# Refactor and Split scripts/code.js

## Summary
Refactored monolithic `scripts/code.js` (2,250 lines) into focused, single-responsibility modules.
Every created module complies with the hard constraint of maximum 300 lines of code.

## Split Modules Breakdown
1. `scripts/logger.js` (35 lines): Structured JSON logger supporting debug, info, warn, error, success levels.
2. `scripts/constants.js` (60 lines): UI class names, default styles, style display names, and SVG icons.
3. `scripts/store.js` (170 lines): Redux architecture store for state management with action dispatchers and observability logging.
4. `scripts/helpers.js` (179 lines): UI utility functions, settings persistence, translations, and theme applicator.
5. `scripts/csl-converter.js` (165 lines): CSL data mapping and type converters.
6. `scripts/csl-loader.js` (108 lines): Asynchronous CSL style and locale fetcher with caching.
7. `scripts/auth.js` (163 lines): OAuth authentication flow and auth view state transitions.
8. `scripts/ui-controls.js` (200 lines): Custom scroller and select dropdown widgets.
9. `scripts/doc-card.js` (94 lines): Document card element DOM constructor.
10. `scripts/citation-selection.js` (286 lines): Citation selection state, pill labels, and edit drawer.
11. `scripts/citation-insert.js` (235 lines): Citation and bibliography insertion logic.
12. `scripts/citation-parser.js` (103 lines): Document text scanning and library citation matching.
13. `scripts/citation-sync.js` (229 lines): Citation refreshing and style updating in document.
14. `scripts/styles-inventory.js` (101 lines): Full CSL styles inventory parser.
15. `scripts/settings-drawer.js` (226 lines): Settings overview and style/language switcher drawers.
16. `scripts/collection-drawer.js` (100 lines): Collection/group drawer views.
17. `scripts/library-view.js` (275 lines): Library search, infinite scroll pagination, and document listing.
18. `scripts/events.js` (212 lines): DOM event listeners and navigation bindings.
19. `scripts/code.js` (178 lines): Main plugin coordinator and lifecycle bootstrap.

## Verification
- All test suites passing (`tests/document.test.js`, `tests/scroll.test.js`, `tests/modules.test.js`).
- Every module verified <= 300 lines of code.
- Script tags updated in `index.html`.
