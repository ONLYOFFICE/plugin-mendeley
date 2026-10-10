# Prepare Linux Release v1.1.5

## Change
- Package compiled x86_64 and aarch64 Rust helpers in `mendeley-linux-installer.tar.gz`; exclude legacy Python helper.
- Verify exact installer archive and lifecycle before publishing.
- Skip Windows build and assets for v1.1.5.
- Bump Linux package version and changelog to 1.1.5.

## Verification
- `cargo build --locked --release` and cross builds for x86_64 and aarch64 succeeded.
- `bash -n install.sh tests/linux-installer.test.sh` passed.
- `bash tests/linux-installer.test.sh` passed against release archive in isolated Docker network.
