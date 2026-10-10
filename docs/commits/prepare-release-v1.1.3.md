# Prepare Release v1.1.3

## Change
Decode HTML entities when refreshing citation text. Bumped plugin and installer metadata to `1.1.3`.

## Verification
- `node tests/document.test.js`
- `node tests/docbuilder.test.js`
- Verified ONLYOFFICE update script receives decoded citation text.
- Verified plugin and installer metadata use `1.1.3`.
