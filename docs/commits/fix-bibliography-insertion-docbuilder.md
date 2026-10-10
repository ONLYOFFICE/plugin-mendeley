# Fix Bibliography Insertion via DocumentBuilder API

## Problem
Inserting a bibliography created an empty Content Control in ONLYOFFICE, displaying only placeholder text "Bibliography".
Root cause: `addContentControl` was attempting a two-step creation (`AddContentControl` followed by `InsertAndReplaceContentControls`). However, `AddContentControl`'s callback does not receive an `InternalId`, triggering an early return that skipped content population and left only the placeholder text.

## Solution
1. Switched `OnlyOfficeAdapter.prototype.addContentControl` to call `InsertAndReplaceContentControls` directly with `Props: { Tag, Lock: 0 }` and DocumentBuilder `Script`, creating and populating the Content Control atomically in one step without relying on placeholder text or callbacks.
2. Kept in-place replacement via `InsertAndReplaceContentControls` targeting `Props: { InternalId }` when the bibliography control already exists.
3. Created `scripts/docbuilder-helper.js` (197 lines) to parse CSL HTML entries into paragraphs and runs with rich text formatting (`oRun.SetItalic(true)`, `oRun.SetBold(true)`), XML entity decoding, dynamic hanging indents, and plain text fallback.
4. Updated `scripts/citation-sync.js` and `scripts/citation-insert.js` to extract CSL style `hangingindent` parameters and pass options for dynamic hanging indents.
5. Added unit test suite in `tests/docbuilder.test.js` and updated `tests/document.test.js` and `tests/modules.test.js`.

## Verification
- `bun test` / `node tests/*.test.js`: All 4 test suites passing (scroll, modules, document, docbuilder).
- Code review sub-agents (Standards and Spec) passed with 0 findings.
