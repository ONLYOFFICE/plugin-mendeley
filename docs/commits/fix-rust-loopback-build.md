# Fix Rust loopback build

- Return thread-safe boxed errors from the Rust server runner so `tiny_http` bind errors convert cleanly.
- Add Cargo dependency lockfile for reproducible helper builds.
- Format Rust source with rustfmt.
