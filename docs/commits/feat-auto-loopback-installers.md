# Add Automatic Mendeley Loopback Installers

## User behavior
- Linux `.deb`/`.rpm` and Windows setup package bundle helper executable and plugin payload for x64/ARM64 build targets.
- Setup selects ONLYOFFICE plugin directory, installs plugin per-user, starts helper immediately, and registers login startup (XDG Autostart on Linux, `HKCU\...\Run` on Windows).
- Uninstall removes selected plugin, startup registration, helper process, and token.
- Existing healthy helper on port 8080 is reused; other port owners produce helper/login errors.

## Security and diagnostics
- Token stored in per-user config directory; POSIX file mode is `0600`.
- CORS allows opaque `null` origin used by Desktop Editors plugin and same-origin loopback callback; rejects other web origins.
- Helper exposes `/health`, `/token`, and controlled `/shutdown`, validates loopback Host, and writes JSON events to per-user `helper.log`.

## Verification
- `python3 -m unittest tests/test_loopback_server.py`: six tests pass.
- `bun test`: all four test files pass.
- Linux per-user install/remove smoke test passes with isolated HOME.
- `python3 -m py_compile` and `bash -n` pass.
- Cross-platform binary/package builds are configured in GitHub Actions; Windows and ARM64 installers cannot be built on this Linux x64 workstation.
