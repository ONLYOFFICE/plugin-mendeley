# Domain Glossary

## Document
The active text document being edited in ONLYOFFICE Document Server containing citations and bibliographies.

## Content Control
A bounded container within the Document created via ONLYOFFICE plugin APIs. Stores rendered human-readable citation text or bibliography HTML in its body, and serialized machine-readable citation metadata in its tag.

## Citation Cluster
An in-text reference marker (e.g., `(Smith, 2020)` or `[1]`) anchored inside a Content Control. Contains one or more citation items with associated locator, prefix, suffix, and display parameters. Serialized into tag format `MENDELEY_CITATION_v3_<base64>`.

## Citation Item
A single cited reference work within a Citation Cluster, containing the source reference identifier, CSL item data, and per-citation formatting flags (such as author suppression or locator page numbers).

## Bibliography
The formatted bibliography block generated from cited works and inserted into the Document, anchored inside a dedicated Content Control with tag `MENDELEY_BIBLIOGRAPHY`.
