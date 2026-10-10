# Update Mendeley Authentication Diagrams

## Change
Updated OAuth sequence and workflow diagrams to reflect Desktop auto-connect, manual Web OAuth callbacks, token storage, API access, and session behavior. Updated README token guidance and regenerated both PNG assets.

## Verification
- `python3 scripts/self_check.py docs/mendeley-auth-sequence.html`: passed.
- `python3 scripts/self_check.py docs/images/mendeley-workflow.html`: passed.
- PNGs verified: `docs/mendeley-auth-sequence.png` (1920 × 1200), `docs/images/mendeley-workflow.png` (2560 × 1440).
