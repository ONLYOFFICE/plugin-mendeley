# Linux Rust packaging

## Change
- Build Debian/RPM packages with release Rust helper instead of PyInstaller Python executable.
- Bundle x86_64 and aarch64 Rust helpers in Linux one-step installer archive and select host architecture during installation.
- Restart helper during installation and stop it during uninstall; verify installer, helper launch, and shutdown behavior.
- Keep Python helper source unchanged.

## Verification

- `cargo test --locked`: 8 Rust tests passed; `cargo build --locked --release` succeeded.
- Linux installer and package lifecycle test passed with the built helper inside isolated network namespace.
- `bun test`, `python3 -m unittest tests/test_loopback_server.py`, and `bash -n` checks passed.
