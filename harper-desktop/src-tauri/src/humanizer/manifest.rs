//! The list of Humanizer model versions we have reviewed and approved.
//!
//! Gramflow never reads the upstream Humanizer repository. It reads this manifest, which we
//! publish ourselves after looking at what changed upstream. A model only reaches users once it
//! is listed here, pinned to an exact revision and checksum.

use super::models::{InstalledModels, Tier};
use serde::{Deserialize, Serialize};

pub const MANIFEST_URL: &str =
    "https://raw.githubusercontent.com/reachneoagent-commits/gramflow/main/humanizer/manifest.json";

const SCHEMA: u32 = 1;
/// The only places an approved model may be downloaded from.
const ALLOWED_HOSTS: [&str; 2] = ["https://huggingface.co/", "https://github.com/"];
/// The manifest is small; refuse anything bigger rather than read it all.
pub const MAX_MANIFEST_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub schema: u32,
    /// Date the manifest was last published, for display.
    pub published: String,
    pub models: Vec<ManifestModel>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestModel {
    /// Model size id, such as "q8".
    pub tier: String,
    /// Label for this release, such as "2026-10-15". Different labels mean a different file.
    pub version: String,
    pub released: String,
    /// What changed, in plain words, shown to the user before they update.
    pub notes: String,
    pub file: String,
    /// Download address, pinned to an exact upstream revision.
    pub url: String,
    pub size_bytes: u64,
    pub sha256: String,
    /// Oldest Gramflow that can use this model.
    pub min_app_version: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ManifestError {
    #[error("The update list isn't valid JSON: {0}")]
    Parse(String),
    #[error("The update list uses a newer format than this version of Gramflow understands.")]
    Schema,
    #[error("The update list has a problem: {0}")]
    Invalid(String),
}

/// Parses and checks a manifest. Anything suspicious rejects the whole file.
pub fn parse(json: &str) -> Result<Manifest, ManifestError> {
    if json.len() > MAX_MANIFEST_BYTES {
        return Err(ManifestError::Invalid("the file is too large".into()));
    }
    let manifest: Manifest =
        serde_json::from_str(json).map_err(|error| ManifestError::Parse(error.to_string()))?;
    if manifest.schema != SCHEMA {
        return Err(ManifestError::Schema);
    }
    for model in &manifest.models {
        validate(model)?;
    }
    Ok(manifest)
}

fn validate(model: &ManifestModel) -> Result<(), ManifestError> {
    let invalid = |reason: String| Err(ManifestError::Invalid(reason));
    if Tier::from_id(&model.tier).is_none() {
        return invalid(format!("unknown model size \"{}\"", model.tier));
    }
    if model.version.trim().is_empty() {
        return invalid(format!("{} has no version", model.tier));
    }
    if model.sha256.len() != 64 || !model.sha256.bytes().all(|b| b.is_ascii_hexdigit()) {
        return invalid(format!("{} has a malformed checksum", model.tier));
    }
    if model.size_bytes == 0 {
        return invalid(format!("{} has no size", model.tier));
    }
    if !ALLOWED_HOSTS.iter().any(|host| model.url.starts_with(host)) {
        return invalid(format!(
            "{} points somewhere we don't download from",
            model.tier
        ));
    }
    if model.file.contains('/') || model.file.contains('\\') || model.file.contains("..") {
        return invalid(format!("{} has an unsafe file name", model.tier));
    }
    if parse_version(&model.min_app_version).is_none() {
        return invalid(format!("{} has a malformed minimum version", model.tier));
    }
    Ok(())
}

/// "3.0.12" becomes (3, 0, 12). A leading "v" is allowed.
pub fn parse_version(version: &str) -> Option<(u64, u64, u64)> {
    let version = version.trim().trim_start_matches(['v', 'V']);
    let mut parts = version.split('.').map(|part| part.parse::<u64>().ok());
    let parsed = (parts.next()??, parts.next()??, parts.next()??);
    parts.next().is_none().then_some(parsed)
}

/// One model that could be updated to a newer approved release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Update {
    pub tier: String,
    pub from_version: String,
    pub to_version: String,
    pub released: String,
    pub notes: String,
    pub size_bytes: u64,
    /// This copy of Gramflow is too old to use the new model; it must update itself first.
    pub needs_newer_app: bool,
}

/// Models the user has installed that have a different approved release.
///
/// `file_size` reports the real size of an installed file. A model downloaded before updates
/// existed has no version record, so it counts as current when its size matches the approved file.
pub fn available_updates(
    manifest: &Manifest,
    app_version: &str,
    installed: &InstalledModels,
    file_size: impl Fn(Tier) -> Option<u64>,
) -> Vec<Update> {
    let app = parse_version(app_version).unwrap_or((0, 0, 0));
    manifest
        .models
        .iter()
        .filter_map(|model| {
            let tier = Tier::from_id(&model.tier)?;
            let size = file_size(tier)?; // not installed: nothing to update
            let record = installed.get(&model.tier);
            let current = match record {
                Some(record) if !record.version.is_empty() => record.version == model.version,
                _ => size == model.size_bytes,
            };
            if current {
                return None;
            }
            Some(Update {
                tier: model.tier.clone(),
                from_version: record
                    .map(|record| record.version.clone())
                    .filter(|version| !version.is_empty())
                    .unwrap_or_else(|| "original".into()),
                to_version: model.version.clone(),
                released: model.released.clone(),
                notes: model.notes.clone(),
                size_bytes: model.size_bytes,
                needs_newer_app: parse_version(&model.min_app_version)
                    .is_some_and(|needed| needed > app),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::humanizer::models::InstalledModel;

    const SHA: &str = "8d7a457b56de6530eaaf0151ccfa7550a4b20dab979737e259da9c63e960e0b0";

    fn model(tier: &str, version: &str, size: u64) -> ManifestModel {
        ManifestModel {
            tier: tier.into(),
            version: version.into(),
            released: "2026-10-15".into(),
            notes: "Better fact fidelity.".into(),
            file: "humanizer-12b-Q8_0.gguf".into(),
            url:
                "https://huggingface.co/jialinyyzz/humanizer/resolve/abc123/humanizer-12b-Q8_0.gguf"
                    .into(),
            size_bytes: size,
            sha256: SHA.into(),
            min_app_version: "3.0.12".into(),
        }
    }

    fn manifest(models: Vec<ManifestModel>) -> Manifest {
        Manifest {
            schema: 1,
            published: "2026-11-11".into(),
            models,
        }
    }

    fn record(version: &str) -> InstalledModel {
        InstalledModel {
            version: version.into(),
            ..Default::default()
        }
    }

    fn json(manifest: &Manifest) -> String {
        serde_json::to_string(manifest).unwrap()
    }

    #[test]
    fn a_good_manifest_parses() {
        let original = manifest(vec![model("q8", "2026-10-15", 100)]);
        assert_eq!(parse(&json(&original)).unwrap(), original);
    }

    #[test]
    fn rejects_bad_json_and_other_formats() {
        assert!(matches!(parse("not json"), Err(ManifestError::Parse(_))));
        let mut newer = manifest(vec![]);
        newer.schema = 2;
        assert_eq!(parse(&json(&newer)), Err(ManifestError::Schema));
    }

    #[test]
    fn rejects_suspicious_entries() {
        let cases = [
            (
                "unknown size",
                ManifestModel {
                    tier: "q99".into(),
                    ..model("q8", "1", 1)
                },
            ),
            (
                "empty version",
                ManifestModel {
                    version: " ".into(),
                    ..model("q8", "1", 1)
                },
            ),
            (
                "short checksum",
                ManifestModel {
                    sha256: "abc".into(),
                    ..model("q8", "1", 1)
                },
            ),
            (
                "non-hex checksum",
                ManifestModel {
                    sha256: "z".repeat(64),
                    ..model("q8", "1", 1)
                },
            ),
            ("zero size", model("q8", "1", 0)),
            (
                "foreign host",
                ManifestModel {
                    url: "https://evil.example/x.gguf".into(),
                    ..model("q8", "1", 1)
                },
            ),
            (
                "plain http",
                ManifestModel {
                    url: "http://huggingface.co/x.gguf".into(),
                    ..model("q8", "1", 1)
                },
            ),
            (
                "path in file name",
                ManifestModel {
                    file: "../x.gguf".into(),
                    ..model("q8", "1", 1)
                },
            ),
            (
                "bad min version",
                ManifestModel {
                    min_app_version: "soon".into(),
                    ..model("q8", "1", 1)
                },
            ),
        ];
        for (name, entry) in cases {
            let result = parse(&json(&manifest(vec![entry])));
            assert!(
                matches!(result, Err(ManifestError::Invalid(_))),
                "{name} should be rejected"
            );
        }
    }

    #[test]
    fn rejects_an_oversized_file() {
        assert!(matches!(
            parse(&" ".repeat(MAX_MANIFEST_BYTES + 1)),
            Err(ManifestError::Invalid(_))
        ));
    }

    #[test]
    fn versions_compare_as_numbers() {
        assert_eq!(parse_version("3.0.12"), Some((3, 0, 12)));
        assert_eq!(parse_version("v3.1.0"), Some((3, 1, 0)));
        assert!(parse_version("3.0.12") > parse_version("3.0.9"));
        assert_eq!(parse_version("3.0"), None);
        assert_eq!(parse_version("3.0.0.1"), None);
        assert_eq!(parse_version("x.y.z"), None);
    }

    #[test]
    fn a_matching_version_needs_no_update() {
        let manifest = manifest(vec![model("q8", "2026-10-15", 100)]);
        let mut installed = InstalledModels::new();
        installed.insert("q8".into(), record("2026-10-15"));
        assert!(available_updates(&manifest, "3.0.12", &installed, |_| Some(100)).is_empty());
    }

    #[test]
    fn a_different_version_is_an_update() {
        let manifest = manifest(vec![model("q8", "2026-11-20", 100)]);
        let mut installed = InstalledModels::new();
        installed.insert("q8".into(), record("2026-10-15"));
        let updates = available_updates(&manifest, "3.0.12", &installed, |_| Some(90));
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].from_version, "2026-10-15");
        assert_eq!(updates[0].to_version, "2026-11-20");
        assert!(!updates[0].needs_newer_app);
    }

    #[test]
    fn an_unrecorded_download_counts_as_current_when_the_size_matches() {
        let manifest = manifest(vec![model("q8", "2026-10-15", 100)]);
        let none = InstalledModels::new();
        assert!(available_updates(&manifest, "3.0.12", &none, |_| Some(100)).is_empty());
        let updates = available_updates(&manifest, "3.0.12", &none, |_| Some(99));
        assert_eq!(updates[0].from_version, "original");
    }

    #[test]
    fn models_that_are_not_installed_are_ignored() {
        let manifest = manifest(vec![model("q8", "2026-11-20", 100)]);
        assert!(
            available_updates(&manifest, "3.0.12", &InstalledModels::new(), |_| None).is_empty()
        );
    }

    #[test]
    fn an_old_app_is_told_to_update_itself_first() {
        let mut entry = model("q8", "2026-11-20", 100);
        entry.min_app_version = "3.1.0".into();
        let manifest = manifest(vec![entry]);
        let updates = available_updates(&manifest, "3.0.12", &InstalledModels::new(), |_| Some(1));
        assert!(updates[0].needs_newer_app);
    }
}
