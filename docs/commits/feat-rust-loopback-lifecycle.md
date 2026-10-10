# Rust loopback server lifecycle

- Added a Cargo binary for the loopback service with `/health`, matching-service reuse, and safe `--stop` handling.
- Added per-user helper log output with structured JSON lifecycle events.
- Kept the Python helper unchanged as the behavior reference.
