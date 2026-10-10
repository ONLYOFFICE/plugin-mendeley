# Fix Release Staging Paths

## Change
Removed nonexistent root-level vendor paths from Linux and Windows staging commands. The required third-party, citation-engine, and Mendeley SDK assets already live under `scripts/`, which both jobs copy.

## Verification
- Confirmed all three referenced asset directories and files are tracked under `scripts/`.
- CI for `v1.1.1` exposed the missing root-level paths.
