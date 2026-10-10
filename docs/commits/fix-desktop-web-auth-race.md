# Desktop-first authentication without web login race

## Cause
In-flight desktop session checks could finish after web OAuth started and replace the web token. When desktop checks found no session, startup kept polling instead of opening web sign-in.

## Change
- Invalidate pending desktop checks when web OAuth begins or user logs out.
- Start web OAuth automatically after initial desktop session lookup fails; retain explicit desktop retry.
- Remove obsolete background desktop polling.
- Cover desktop-first success, automatic web fallback, and late desktop response with startup integration test.
- Update authentication instructions and changelog.

## Verification
- `node tests/auth-race.test.js` passed after reproducing the original race and missing fallback before the fixes.
- All `tests/*.test.js` passed.
