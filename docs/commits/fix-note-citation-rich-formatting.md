# Preserve CSL Formatting in Note Citations

## Problem
Note-style citations were inserted beside the footnote number but as plain text. CSL emphasis such as italic journal titles was lost because the footnote path stripped HTML and used `ApiInlineLvlSdt.AddText()`.

## Fix
1. Pass raw CSL-rendered citation HTML into `addNoteCitation`.
2. Reuse `DocBuilderHelper.parseHtmlToRuns` to derive text runs and italic/bold state.
3. Create `ApiRun` per CSL run and apply `SetItalic(true)` / `SetBold(true)` before adding each run to the inline Content Control.
4. Keep raw rendered HTML in `InMemoryAdapter` so regression test verifies formatting source survives note insertion.

## Verification
- `bun test`: all 4 test suites pass.
