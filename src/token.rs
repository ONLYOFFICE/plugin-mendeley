use serde_json::{json, Value};
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_FILE_ID: AtomicU64 = AtomicU64::new(0);

pub(crate) fn token_file_path() -> PathBuf {
    let root = if cfg!(windows) {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                std::env::var_os("USERPROFILE")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("AppData/Roaming")
            })
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                std::env::var_os("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".config")
            })
    };
    root.join("mendeley-onlyoffice").join("active-token.json")
}

pub(crate) fn write_token(path: &Path, token: &str) -> std::io::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let mut temporary_path = None;
    let mut file = None;
    for _ in 0..100 {
        let id = TEMP_FILE_ID.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(".token-{}-{id}", std::process::id()));
        match create_private_file(&candidate) {
            Ok(created) => {
                temporary_path = Some(candidate);
                file = Some(created);
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    let temporary_path = temporary_path.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "could not create token file",
        )
    })?;
    let result = (|| {
        let mut file = file.expect("temporary token file exists");
        serde_json::to_writer(&mut file, &json!({"token": token}))
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        use std::io::Write;
        file.flush()?;
        file.sync_all()?;
        replace_file(&temporary_path, path)?;
        set_private_permissions(path)?;
        Ok(())
    })();
    if temporary_path.exists() {
        let _ = fs::remove_file(&temporary_path);
    }
    result
}

#[cfg(unix)]
fn create_private_file(path: &Path) -> std::io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
}

#[cfg(not(unix))]
fn create_private_file(path: &Path) -> std::io::Result<File> {
    OpenOptions::new().write(true).create_new(true).open(path)
}

#[cfg(unix)]
fn set_private_permissions(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn set_private_permissions(_: &Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(windows)]
fn replace_file(source: &Path, destination: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "Kernel32")]
    extern "system" {
        fn MoveFileExW(existing: *const u16, new: *const u16, flags: u32) -> i32;
    }
    let source: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    let destination: Vec<u16> = destination
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    if unsafe { MoveFileExW(source.as_ptr(), destination.as_ptr(), 0x1 | 0x8) } == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn replace_file(source: &Path, destination: &Path) -> std::io::Result<()> {
    fs::rename(source, destination)
}

pub(crate) fn read_token(path: &Path) -> Option<String> {
    let mut contents = String::new();
    File::open(path).ok()?.read_to_string(&mut contents).ok()?;
    let value = serde_json::from_str::<Value>(&contents).ok()?;
    let token = value.get("token")?.as_str()?;
    (!token.is_empty()).then(|| token.to_owned())
}

pub(crate) fn read_token_or_recover(
    path: &Path,
    recover: impl FnOnce() -> Option<String>,
    logger: &crate::logger::Logger,
) -> Option<String> {
    if let Some(token) = read_token(path) {
        return Some(token);
    }
    let token = recover()?;
    match write_token(path, &token) {
        Ok(()) => logger.event("success", "mendeley.token_persisted", json!({})),
        Err(error) => logger.event(
            "error",
            "mendeley.token_persist_failed",
            json!({"error": error.to_string()}),
        ),
    }
    Some(token)
}

pub(crate) fn clear_token(path: &Path) -> std::io::Result<()> {
    match fs::remove_file(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}
