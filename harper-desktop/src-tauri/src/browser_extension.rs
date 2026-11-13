//! Detects whether the Gramflow browser extension is installed in each browser profile.
//!
//! Chromium browsers keep one extension list per profile. Unpacked extensions are recorded
//! with the folder they were loaded from, so we recognise ours by that folder name.

use serde::Serialize;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Folder name `prepare_browser_extension` copies the extension into.
pub const EXTENSION_FOLDER_NAME: &str = "Gramflow Browser Extension";

/// Folder the Firefox build of the extension is copied into.
pub const FIREFOX_EXTENSION_FOLDER_NAME: &str = "Gramflow Firefox Extension";

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
    /// Chrome's id for the extension in this profile, used to open its details page.
    pub extension_id: Option<String>,
    /// Version of the extension files in the folder the profile loads from.
    pub folder_version: Option<String>,
    /// Version that ships with this copy of Gramflow.
    pub bundled_version: String,
}

/// What a profile's preferences say about the Gramflow extension.
#[derive(Debug, PartialEq, Eq)]
struct ExtensionRecord {
    id: String,
    path: PathBuf,
    enabled: bool,
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
            let record = extension_record(&root.join(dir));
            let folder_version = record
                .as_ref()
                .and_then(|record| folder_version(&record.path));
            BrowserProfileStatus {
                browser: browser.to_owned(),
                profile_dir: dir.clone(),
                profile_name: info["name"].as_str().unwrap_or(dir).to_owned(),
                email: info["user_name"]
                    .as_str()
                    .filter(|email| !email.is_empty())
                    .map(str::to_owned),
                installed: record.is_some(),
                enabled: record.as_ref().is_some_and(|record| record.enabled),
                extension_id: record.map(|record| record.id),
                folder_version,
                bundled_version: env!("CARGO_PKG_VERSION").to_owned(),
            }
        })
        .collect();

    profiles.sort_by(|a, b| a.profile_dir.cmp(&b.profile_dir));
    profiles
}

/// Finds the Gramflow extension in one profile directory.
fn extension_record(profile: &Path) -> Option<ExtensionRecord> {
    // Chrome writes extension records to `Secure Preferences`; older builds used `Preferences`.
    ["Secure Preferences", "Preferences"]
        .iter()
        .filter_map(|file| read_json(&profile.join(file)))
        .find_map(|prefs| extension_record_from_prefs(&prefs))
}

fn extension_record_from_prefs(prefs: &Value) -> Option<ExtensionRecord> {
    prefs["extensions"]["settings"]
        .as_object()?
        .iter()
        .find(|(_, entry)| {
            entry["path"].as_str().is_some_and(|path| {
                Path::new(path)
                    .file_name()
                    .is_some_and(|name| name == EXTENSION_FOLDER_NAME)
            })
        })
        .map(|(id, entry)| {
            let has_disable_reasons = entry["disable_reasons"]
                .as_array()
                .is_some_and(|reasons| !reasons.is_empty())
                || entry["disable_reasons"]
                    .as_i64()
                    .is_some_and(|bits| bits != 0);

            ExtensionRecord {
                id: id.clone(),
                path: PathBuf::from(entry["path"].as_str().unwrap_or_default()),
                enabled: entry["state"].as_i64() != Some(0) && !has_disable_reasons,
            }
        })
}

/// Reads the version from the extension's `manifest.json` in its folder.
fn folder_version(folder: &Path) -> Option<String> {
    read_json(&folder.join("manifest.json"))?["version"]
        .as_str()
        .map(str::to_owned)
}

fn read_json(path: &Path) -> Option<Value> {
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
            "/Users/a/Documents/Gramflow Browser Extension",
            json!({"disable_reasons": []}),
        );
        let record = extension_record_from_prefs(&p).unwrap();
        assert!(record.enabled);
        assert_eq!(record.id, "abc");
        assert_eq!(
            record.path,
            PathBuf::from("/Users/a/Documents/Gramflow Browser Extension")
        );
    }

    #[test]
    fn detects_disabled_extension() {
        let p = prefs("/x/Gramflow Browser Extension", json!({"state": 0}));
        assert!(!extension_record_from_prefs(&p).unwrap().enabled);
        let p = prefs(
            "/x/Gramflow Browser Extension",
            json!({"disable_reasons": [1]}),
        );
        assert!(!extension_record_from_prefs(&p).unwrap().enabled);
    }

    #[test]
    fn ignores_other_extensions() {
        let p = prefs("/x/Some Other Extension", json!({}));
        assert_eq!(extension_record_from_prefs(&p), None);
        assert_eq!(extension_record_from_prefs(&json!({})), None);
    }

    #[test]
    fn reads_folder_version_from_manifest() {
        let dir = std::env::temp_dir().join("gramflow-ext-version-test");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("manifest.json"), r#"{"version": "3.0.3"}"#).unwrap();

        assert_eq!(folder_version(&dir), Some("3.0.3".to_owned()));
        assert_eq!(folder_version(&dir.join("missing")), None);
    }
}
