# Fix Mendeley desktop refresh-cookie token lookup

## Cause
The loopback helper referenced `urllib.request` without importing it. Refresh-cookie extraction swallowed the resulting `AttributeError` and reported no token. Direct `accessToken` and cached-token paths did not exercise this branch.

## Change
- Import `urllib.request` explicitly and use its stable module binding.
- Add regression coverage for refresh-cookie extraction when `urllib.request` is absent from the `urllib` package namespace.

## Verification
- `python3 tests/test_loopback_server.py`: 12 tests passed.
- `node tests/auth-race.test.js` and `node tests/auth-login.test.js` passed.
