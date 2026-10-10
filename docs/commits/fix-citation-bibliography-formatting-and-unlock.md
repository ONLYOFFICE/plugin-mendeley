# Fix Citation and Bibliography Formatting

## Problem
- Mendeley source and series were both mapped to CSL `container-title`, duplicating journal names.
- Bibliography runs omitted explicit false italic/bold formatting, allowing editor formatting to leak into plain runs.
- Content controls used `Lock: 0`, which does not provide full edit-and-delete access in ONLYOFFICE's plugin API. Existing Mendeley controls also needed migration.
- Style change launched citation and bibliography refresh writes without awaiting completion.

## Solution
- Map source to `container-title` and series to `collection-title`.
- Explicitly set italic and bold for every bibliography run based on CSL markup.
- Set new citation and bibliography controls to `Lock: 3`, set note controls to `unlocked`, and unlock existing Mendeley controls at plugin startup.
- Await citation and bibliography refresh writes before closing style selection.
- Regression checks cover mapping, run formatting defaults, and full access lock values.

## Verification
- `node tests/modules.test.js`
- `node tests/docbuilder.test.js`
- `node tests/document.test.js`
- `node --check scripts/document.js && node --check scripts/code.js`
