use crate::logger::Logger;
use rusqlite::Connection;
use serde_json::Value;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const REFRESH_URL: &str = "https://www.mendeley.com/reference-manager-desktop/refresh-token";
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) MendeleyReferenceManager/2.145.0 Chrome/144.0.7559.220 Electron/40.6.1 Safari/537.36";

fn home() -> PathBuf {
    let home = if cfg!(windows) {
        env::var_os("USERPROFILE").or_else(|| env::var_os("HOME"))
    } else {
        env::var_os("HOME").or_else(|| env::var_os("USERPROFILE"))
    };
    home.map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn add_dir(path: impl AsRef<Path>, paths: &mut Vec<PathBuf>, seen: &mut HashSet<PathBuf>) {
    let path = path.as_ref();
    if let Ok(path) = fs::canonicalize(path) {
        if path.is_dir() && seen.insert(path.clone()) {
            paths.push(path);
        }
    }
}

pub(crate) fn find_storage_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut seen = HashSet::new();
    if let Some(path) = env::var_os("MENDELEY_USER_DATA_DIR") {
        add_dir(path, &mut paths, &mut seen);
    }
    let home = home();
    match env::consts::OS {
        "windows" => {
            let roaming = env::var_os("APPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join("AppData/Roaming"));
            let local = env::var_os("LOCALAPPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join("AppData/Local"));
            for base in [&roaming, &local] {
                add_dir(
                    base.join("Mendeley Reference Manager"),
                    &mut paths,
                    &mut seen,
                );
                add_dir(
                    base.join("mendeley-reference-manager"),
                    &mut paths,
                    &mut seen,
                );
            }
            if let Ok(packages) = fs::read_dir(local.join("Packages")) {
                for package in packages
                    .flatten()
                    .filter(|entry| entry.file_name().to_string_lossy().contains("Mendeley"))
                {
                    add_dir(
                        package
                            .path()
                            .join("LocalCache/Roaming/Mendeley Reference Manager"),
                        &mut paths,
                        &mut seen,
                    );
                }
            }
        }
        "macos" => {
            add_dir(
                home.join("Library/Application Support/Mendeley Reference Manager"),
                &mut paths,
                &mut seen,
            );
            add_dir(
                home.join("Library/Containers/com.elsevier.MendeleyReferenceManager/Data/Library/Application Support/Mendeley Reference Manager"),
                &mut paths,
                &mut seen,
            );
        }
        _ => {
            let xdg = env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home.join(".config"));
            add_dir(
                xdg.join("Mendeley Reference Manager"),
                &mut paths,
                &mut seen,
            );
            add_dir(
                home.join(".config/Mendeley Reference Manager"),
                &mut paths,
                &mut seen,
            );
            add_dir(
                home.join(".var/app/com.elsevier.MendeleyReferenceManager/config/Mendeley Reference Manager"),
                &mut paths,
                &mut seen,
            );
            for suffix in ["current", "common"] {
                add_dir(
                    home.join(format!(
                        "snap/mendeley-reference-manager/{suffix}/.config/Mendeley Reference Manager"
                    )),
                    &mut paths,
                    &mut seen,
                );
            }
            find_process_storage_paths(&mut paths, &mut seen);
        }
    }
    paths
}

#[cfg(target_os = "linux")]
fn find_process_storage_paths(paths: &mut Vec<PathBuf>, seen: &mut HashSet<PathBuf>) {
    let Ok(entries) = fs::read_dir("/proc") else {
        return;
    };
    for entry in entries.flatten().filter(|entry| {
        entry
            .file_name()
            .to_string_lossy()
            .bytes()
            .all(|byte| byte.is_ascii_digit())
    }) {
        let Ok(command) = fs::read(entry.path().join("cmdline")) else {
            continue;
        };
        if !command
            .to_ascii_lowercase()
            .windows(8)
            .any(|part| part == b"mendeley")
        {
            continue;
        }
        for argument in command.split(|byte| *byte == 0) {
            if let Some(path) = argument.strip_prefix(b"--user-data-dir=") {
                add_dir(String::from_utf8_lossy(path).as_ref(), paths, seen);
            }
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn find_process_storage_paths(_: &mut Vec<PathBuf>, _: &mut HashSet<PathBuf>) {}

fn refresh_token(cookie_header: &str) -> Option<String> {
    let response = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .ok()?
        .post(REFRESH_URL)
        .header("Content-Type", "application/json")
        .header("Cookie", cookie_header)
        .header("User-Agent", USER_AGENT)
        .body("{}")
        .send()
        .ok()?;
    if response.status().as_u16() != 200 {
        return None;
    }
    let value = response.json::<Value>().ok()?;
    ["access_token", "accessToken", "token"]
        .iter()
        .filter_map(|key| value.get(key).and_then(Value::as_str))
        .find(|token| token.chars().count() > 20)
        .map(str::to_owned)
}

fn extract_cookie_token(path: &Path) -> Option<String> {
    let connection =
        Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).ok()?;
    let mut statement = connection
        .prepare("SELECT name, value, encrypted_value FROM cookies")
        .ok()?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0).unwrap_or_default(),
                row.get::<_, String>(1).unwrap_or_default(),
                row.get::<_, Vec<u8>>(2).unwrap_or_default(),
            ))
        })
        .ok()?;
    let mut cookies = Vec::new();
    for row in rows.flatten() {
        let (name, mut value, encrypted) = row;
        if value.is_empty() && env::consts::OS == "windows" && !encrypted.is_empty() {
            value = crate::windows_cookie::decrypt(&encrypted).unwrap_or_default();
        }
        if name == "accessToken" && value.chars().count() > 20 {
            return Some(value);
        }
        if !value.is_empty() {
            cookies.push(format!("{name}={value}"));
        }
    }
    let cookie_header = cookies.join("; ");
    if cookie_header.is_empty() {
        None
    } else {
        refresh_token(&cookie_header)
    }
}
fn cache_token(root: &Path) -> Option<String> {
    let mut pending = vec![root.join("Service Worker/CacheStorage")];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
                pending.push(path);
                continue;
            }
            let Ok(content) = fs::read(path) else {
                continue;
            };
            let mut offset = 0;
            while let Some(found) = content[offset..]
                .windows(6)
                .position(|bytes| bytes == b"Bearer")
            {
                let mut start = offset + found + 6;
                while content
                    .get(start)
                    .is_some_and(|byte| byte.is_ascii_whitespace())
                {
                    start += 1;
                }
                if start == offset + found + 6 {
                    offset = start;
                    continue;
                }
                let end = content[start..]
                    .iter()
                    .position(|byte| !byte.is_ascii_alphanumeric() && !b"_-.+=".contains(byte))
                    .map(|length| start + length)
                    .unwrap_or(content.len());
                if end - start > 20 {
                    return std::str::from_utf8(&content[start..end])
                        .ok()
                        .map(str::to_owned);
                }
                offset = end;
                if offset >= content.len() {
                    break;
                }
            }
        }
    }
    None
}

pub(crate) fn get_token(logger: &Logger) -> Option<String> {
    let paths = find_storage_paths();
    logger.event("debug", "mendeley.storage_paths_scanned", serde_json::json!({"count": paths.len(), "paths": paths.iter().map(|path| path.display().to_string()).collect::<Vec<_>>()}));
    for directory in &paths {
        for relative in [Path::new("Cookies"), Path::new("Network/Cookies")] {
            let cookie = directory.join(relative);
            if cookie.is_file() {
                if let Some(token) = extract_cookie_token(&cookie) {
                    logger.event(
                        "info",
                        "mendeley.token_extracted",
                        serde_json::json!({"source": "cookies", "path": cookie}),
                    );
                    return Some(token);
                }
            }
        }

        if let Some(token) = cache_token(directory) {
            logger.event(
                "info",
                "mendeley.token_extracted",
                serde_json::json!({"source": "cache", "path": directory}),
            );
            return Some(token);
        }
    }
    logger.event(
        "warn",
        "mendeley.token_not_found",
        serde_json::json!({"scanned_count": paths.len()}),
    );
    None
}
#[cfg(test)]
#[path = "mendeley_tests.rs"]
mod tests;
