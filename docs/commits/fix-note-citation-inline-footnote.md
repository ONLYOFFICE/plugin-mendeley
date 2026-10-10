# Fix Note/Footnote Citation Inline Placement

## Problem
When inserting a citation with a note/footnote style (e.g. Chicago Notes-Bibliography), the citation text was placed on a new line/paragraph below the footnote number `1`.
Root cause: `addFootnote()` created the footnote paragraph with the number `1`, but then `addContentControl()` used `InsertContent([oPara])` which inserted a second paragraph into the footnote rather than appending inline within the footnote's first paragraph.

## Solution
1. Added `OnlyOfficeAdapter.prototype.addNoteCitation(tag, cleanText)` using DocumentBuilder API:
   - Calls `oDoc.AddFootnote()`.
   - Accesses the footnote's initial paragraph via `oDoc.GetFootnotesFirstParagraphs()`.
   - Creates an inline content control via `Api.CreateInlineLvlSdt()`, sets the tag and text.
   - Appends it directly to the footnote's paragraph via `fnPara.AddInlineLvlSdt(sdt)`.
2. Updated `DocumentModule.prototype.insertCitation` to route note-style citations through `adapter.addNoteCitation`.
3. Updated `InMemoryAdapter` with `addNoteCitation` for unit test parity.

## Verification
- All 4 test suites passing (`bun test`).
