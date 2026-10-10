# 04: Switch Linux packaging to Rust helper

**What to build:** Linux packages install and launch the Rust helper for user installation, service startup, and shutdown.

**Blocked by:** 03: Port Mendeley token recovery

**Status:** done

**Python functions in scope:** None; this ticket ports build, package, and installer integration.

- [x] Linux package build produces and stages the Rust helper instead of building a Python executable.
- [x] User installer and service lifecycle launch and stop the Rust helper.
- [x] Linux installer and packaging checks verify the installed helper and service behavior.
- [x] Existing Python implementation remains in the repository unchanged; packaging cutover does not delete it.