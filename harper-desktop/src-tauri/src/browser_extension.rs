//! Detects whether the Flowgram browser extension is installed in each browser profile.
//!
//! Chromium browsers keep one extension list per profile. Unpacked extensions are recorded
//! with the folder they were loaded from, so we recognise ours by that folder name.

use serde::Serialize;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Folder name `prepare_browser_extension` copies the extension into.
pub const EXTENSION_FOLDER_NAME: &str = "Flowgram Browser Extension";

/// Folder the Firefox build of the extension is copied into.
pub const FIREFOX_EXTENSION_FOLDER_NAME: &str = "Flowgram Firefox Extension";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BrowserProfileStatus {
    /// `chrome`, `edge` or `brave`.
    pub browser: String,
    /// Directory name Chrome uses for the profile, e.g. `Default` or `Profile 1`.
    pub profile_dir: String,
    pub profile_name: String,
    pub email: Option<String>,
    pub installed: bool,
    pub enabled: bool,
}

struct BrowserLocation {
    id: &'static str,
    support_dir: &'static str,
}

const BROWSERS: &[BrowserLocation] = &[
    BrowserLocation {
        id: "chrome",
        support_dir: "Google/Chrome",
    },
    BrowserLocation {
        id: "edge",
        support_dir: "Microsoft Edge",
    },
    BrowserLocation {
        id: "brave",
        support_dir: "BraveSoftware/Brave-Browser",
    },
];

/// Lists every profile of every supported browser installed on this machine.
pub fn detect_profiles() -> Vec<BrowserProfileStatus> {
    if !cfg!(target_os = "macos") {
        return Vec::new();
    }

    let Some(support) = dirs::data_dir() else {
        return Vec::new();
    };

    BROWSERS
        .iter()
        .flat_map(|browser| profiles_in(browser.id, &support.join(browser.support_dir)))
        .collect()
}

fn profiles_in(browser: &str, root: &Path) -> Vec<BrowserProfileStatus> {
    let Some(local_state) = read_json(&root.join("Local State")) else {
        return Vec::new();
    };
    let Some(cache) = local_state["profile"]["info_cache"].as_object() else {
        return Vec::new();
    };

    let mut profiles: Vec<BrowserProfileStatus> = cache
        .iter()
        .map(|(dir, info)| {
            let (installed, enabled) = extension_state(&root.join(dir));
            BrowserProfileStatus {
                browser: browser.to_owned(),
                profile_dir: dir.clone(),
                profile_name: info["name"].as_str().unwrap_or(dir).to_owned(),
                email: info["user_name"]
                    .as_str()
                    .filter(|email| !email.is_empty())
                    .map(str::to_owned),
                installed,
                enabled,
            }
        })
        .collect();

    profiles.sort_by(|a, b| a.profile_dir.cmp(&b.profile_dir));
    profiles
}

/// Returns `(installed, enabled)` for the extension in one profile directory.
fn extension_state(profile: &Path) -> (bool, bool) {
    // Chrome writes extension records to `Secure Preferences`; older builds used `Preferences`.
    ["Secure Preferences", "Preferences"]
        .iter()
        .filter_map(|file| read_json(&profile.join(file)))
        .find_map(|prefs| extension_state_from_prefs(&prefs))
        .unwrap_or((false, false))
}

fn extension_state_from_prefs(prefs: &Value) -> Option<(bool, bool)> {
    prefs["extensions"]["settings"]
        .as_object()?
        .values()
        .find(|entry| {
            entry["path"].as_str().is_some_and(|path| {
                Path::new(path)
                    .file_name()
                    .is_some_and(|n| n == EXTENSION_FOLDER_NAME)
            })
        })
        .map(|entry| {
            let has_disable_reasons = entry["disable_reasons"]
                .as_array()
                .is_some_and(|reasons| !reasons.is_empty())
                || entry["disable_reasons"]
                    .as_i64()
                    .is_some_and(|bits| bits != 0);
            let enabled = entry["state"].as_i64() != Some(0) && !has_disable_reasons;
            (true, enabled)
        })
}

fn read_json(path: &PathBuf) -> Option<Value> {
    serde_json::from_slice(&std::fs::read(path).ok()?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn prefs(path: &str, extra: Value) -> Value {
        let mut entry = json!({ "path": path });
        entry
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        json!({ "extensions": { "settings": { "abc": entry } } })
    }

    #[test]
    fn detects_enabled_unpacked_extension() {
        let p = prefs(
            "/Users/a/Documents/Flowgram Browser Extension",
            json!({"disable_reasons": []}),
        );
        assert_eq!(extension_state_from_prefs(&p), Some((true, true)));
    }

    #[test]
    fn detects_disabled_extension() {
        let p = prefs("/x/Flowgram Browser Extension", json!({"state": 0}));
        assert_eq!(extension_state_from_prefs(&p), Some((true, false)));
        let p = prefs(
            "/x/Flowgram Browser Extension",
            json!({"disable_reasons": [1]}),
        );
        assert_eq!(extension_state_from_prefs(&p), Some((true, false)));
    }

    #[test]
    fn ignores_other_extensions() {
        let p = prefs("/x/Some Other Extension", json!({}));
        assert_eq!(extension_state_from_prefs(&p), None);
        assert_eq!(extension_state_from_prefs(&json!({})), None);
    }
}
