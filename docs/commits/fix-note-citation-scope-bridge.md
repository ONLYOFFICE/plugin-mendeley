# Fix Note Citation Scope Bridge

## Problem
Note citation insertion created footnote number `1` but no citation text.

## Root Cause
`callCommand` executes in ONLYOFFICE editor frame and cannot capture plugin-local `tag` or `cleanText`. Previous code passed `{ tag, text }` as a fifth `callCommand` argument, but that argument is unsupported. `Asc.scope.tag` and `Asc.scope.text` were therefore unset inside editor frame.

## Fix
Set `window.Asc.scope.noteTag` and `window.Asc.scope.noteText` before `callCommand`. Editor script reads those values to create tagged `ApiInlineLvlSdt` in footnote first paragraph. If inline content control API fails, it appends plain text to same paragraph.

## Verification
- `bun test`: all 4 test suites pass.
