use serde_json::{json, Value};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Logger(Mutex<File>);

impl Logger {
    pub fn new() -> std::io::Result<Self> {
        let path = log_file_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
        }
        Ok(Self(Mutex::new(file)))
    }

    pub fn event(&self, level: &str, event: &str, data: Value) {
        let record = json!({"level": level, "event": event, "data": data});
        if let Ok(mut file) = self.0.lock() {
            let _ = writeln!(file, "{record}");
        }
    }
}

fn log_file_path() -> std::io::Result<PathBuf> {
    let root = if cfg!(windows) {
        if let Some(appdata) = env::var_os("APPDATA") {
            PathBuf::from(appdata)
        } else {
            env::var_os("USERPROFILE")
                .map(PathBuf::from)
                .unwrap_or(env::current_dir()?)
                .join("AppData/Roaming")
        }
    } else if let Some(config) = env::var_os("XDG_CONFIG_HOME") {
        PathBuf::from(config)
    } else {
        env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or(env::current_dir()?)
            .join(".config")
    };
    Ok(root.join("mendeley-onlyoffice").join("helper.log"))
}
