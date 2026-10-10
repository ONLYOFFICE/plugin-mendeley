# Filter Release Assets

## Change
Limited Linux workflow artifacts to the generated `.deb` and `.rpm` installers, preventing staged plugin files and duplicate basenames from being published as release assets.

## Verification
- Confirmed Linux artifact paths select only `dist/*.deb` and `dist/*.rpm`; Windows artifacts select only installer `.exe` files.
