# 0001. Deep Document Module and Seam

## Context
ONLYOFFICE plugin code directly mixed DOM manipulation, CSL formatting, Base64 tag serialization, and host document editor scripting (`window.Asc.plugin.executeMethod` and `window.Asc.plugin.callCommand`). This prevented automated testing of citation persistence outside a live ONLYOFFICE Document Server and scattered citation metadata encoding across five call sites.

## Decision
Introduce a deep `DocumentModule` in `scripts/document.js` behind a granular, Promise-based interface (`insertCitation`, `getCitations`, `updateCitationText`, `insertBibliography`, `getBibliography`, `updateBibliographyHtml`, `unlinkAll`).

Place an explicit seam at `DocumentAdapter` with two adapters:
1. `OnlyOfficeAdapter`: interacts with `window.Asc.plugin` and host document commands.
2. `InMemoryAdapter`: stateful in-memory control store used by automated unit tests.

The module encapsulates all `MENDELEY_CITATION_v3_` Base64 tag serialization, footnote mechanics, and ONLYOFFICE document script templates. Callers only send and receive plain domain objects (`CitationCluster`, `CitationItem`, `Bibliography`). Support for legacy tag formats is omitted per project simplicity standards.

## Status
Accepted
