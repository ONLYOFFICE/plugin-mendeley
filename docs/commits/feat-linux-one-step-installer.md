# One-step Linux installer

## Change
- Install plugin and loopback helper in one run.
- Detect native, Flatpak, Snap, and XDG data paths; support `ONLYOFFICE_PLUGIN_DIR` and interactive custom-path selection.
- Restart the helper after replacing its script so fixes take effect.
- Build and publish `mendeley-linux-installer.tar.gz` alongside the manual `.plugin` asset.
- Clarify manual plugin-only installation requirements.

## Verification
- `bash tests/linux-installer.test.sh` covers native, Flatpak, Snap, custom-path, and unknown-path cases in isolated home directories.
