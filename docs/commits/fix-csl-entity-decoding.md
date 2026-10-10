# Decode CSL Entities Before Citation Insertion

## Problem
APA 6th edition citations displayed literal `&#38;` instead of ampersand: `(Furlanetto, Sedrez, Candotti, &#38; Loss, 2016)`.

## Root Cause
citeproc emits HTML/XML entities in rendered CSL text. Normal citation path stripped HTML tags but did not decode entities before passing text to ONLYOFFICE.

## Fix
`DocumentModule.insertCitation` now decodes entities through `DocBuilderHelper.decodeEntities` after stripping HTML and before creating the citation Content Control or add-in field.

## Verification
- Added regression assertion for `&#38;` decoding to `tests/document.test.js`.
- `bun test`: all 4 test suites pass.
