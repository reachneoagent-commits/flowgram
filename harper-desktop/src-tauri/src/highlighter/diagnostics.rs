//! A small log file for the highlighter overlay. The overlay runs in its own process with no
//! console, so without this a user whose highlights misbehave has nothing to send us.

use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_BYTES: u64 = 256 * 1024;

pub fn log_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("harper-desktop").join("highlighter.log"))
}

/// Appends one line, with the time as seconds since the Unix epoch. Starts a fresh file once it
/// grows past 256 KB. Logging never fails the caller.
pub fn log(message: impl AsRef<str>) {
    let message = message.as_ref();
    eprintln!("{message}");

    let Some(path) = log_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if std::fs::metadata(&path).is_ok_and(|meta| meta.len() > MAX_BYTES) {
        let _ = std::fs::remove_file(&path);
    }
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0);
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = writeln!(file, "[{seconds}] {message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_log_lives_in_the_gramflow_config_folder() {
        let path = log_path().unwrap();
        assert_eq!(path.file_name().unwrap(), "highlighter.log");
        assert_eq!(
            path.parent().unwrap().file_name().unwrap(),
            "harper-desktop"
        );
    }
}
