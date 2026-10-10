# 05: Switch Windows packaging to Rust helper

**What to build:** Windows installer builds, installs, starts, and stops the Rust helper.

**Blocked by:** 03: Port Mendeley token recovery

**Status:** completed

**Python functions in scope:** None; this ticket ports build, package, and installer integration.

- [x] Windows package build produces the Rust helper for x64 and ARM64.
- [x] Installer startup and uninstall shutdown invoke the Rust helper.
- [x] Windows packaging checks verify the produced executable and installer integration.
- [x] Existing Python implementation remains in the repository unchanged; packaging cutover does not delete it.