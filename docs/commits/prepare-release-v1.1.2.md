# Prepare Release v1.1.2

## Change
Fixed citation insertion to use inline content controls. Bumped plugin and installer metadata to `1.1.2`, added `.plugin` packaging to GitHub Actions release assets, and enabled generated release notes.

## Verification
- `node tests/document.test.js`
- `node tests/docbuilder.test.js`
- `node tests/modules.test.js`
- `node tests/library-pagination.test.js`
- `node tests/scroll.test.js`
- `node tests/auth-login.test.js`
- Verified `config.json`, GitHub Actions, Windows installer, and changelog use `1.1.2`.
