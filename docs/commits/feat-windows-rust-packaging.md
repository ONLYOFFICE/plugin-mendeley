# Windows Rust packaging

## Change
- Build native x64 and ARM64 Windows Rust helpers in package workflow and stage them in the Inno Setup payload.
- Start the helper after install, including silent installs, and stop it during uninstall.
- Add a Windows installer lifecycle check with JSON event logs, repeated health-state polling, and shutdown verification.
- Keep the Python helper source in the repository and staged plugin payload.

## Verification
- Windows installer lifecycle check runs in each Windows architecture workflow job.
