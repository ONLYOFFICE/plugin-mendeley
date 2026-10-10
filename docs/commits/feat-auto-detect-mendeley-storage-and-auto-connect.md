# Auto-Detect Mendeley Storage Paths and Polling Connect

## User behavior
- Helper dynamically scans Mendeley Reference Manager storage across custom paths, running processes (`/proc` on Linux), environment variables, native OS paths, Flatpak, and Snap.
- Helper extracts access tokens via desktop session cookies (`refresh-token` API) and offline cache fallback (`Service Worker/CacheStorage`), matching modern Mendeley Desktop architecture.
- ONLYOFFICE plugin initiates background polling when on login/config screens to automatically detect running Mendeley Desktop sessions without requiring manual clicks.

## Diagnostics and logs
- Emits structured JSON events (`mendeley.storage_paths_scanned`, `mendeley.token_extracted`, `mendeley.token_not_found`).
- Retains legacy `accessToken` cookie fallback.

## Verification
- `python3 tests/test_loopback_server.py`: 11 tests pass.
- All Node.js tests (`auth-login`, `docbuilder`, `document`, `library-pagination`, `modules`, `scroll`) pass.
- Live `/token` verification on running desktop instance: returns valid 200 token.
