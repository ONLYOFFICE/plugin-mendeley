# Fix nFPM Staging Expansion

## Change
Enabled nFPM environment expansion for staged Linux package sources using `${STAGE}`.

## Verification
- CI exposed literal `${STAGE}` paths in nFPM glob resolution.
- nFPM configuration now enables `expand: true` for each staged source.
