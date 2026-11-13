//! The Humanizer model catalog: which sizes exist, what each needs, and where each lives.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

const GB: u64 = 1_000_000_000;

/// One downloadable model size. Values come from the Humanizer model card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    Q8,
    Q6,
    Q4,
    Q3,
    Q2,
}

pub struct TierInfo {
    pub tier: Tier,
    pub id: &'static str,
    pub label: &'static str,
    pub blurb: &'static str,
    pub file: &'static str,
    /// Approximate download size, in bytes.
    pub size_bytes: u64,
    /// Total system memory the model needs, in gigabytes.
    pub min_ram_gb: u64,
}

/// Largest and best first, so the first allowed entry is the recommendation.
pub const TIERS: [TierInfo; 5] = [
    TierInfo {
        tier: Tier::Q8,
        id: "q8",
        label: "Best quality",
        blurb: "Highest fidelity. Needs a lot of memory.",
        file: "humanizer-12b-Q8_0.gguf",
        size_bytes: 127 * GB / 10,
        min_ram_gb: 32,
    },
    TierInfo {
        tier: Tier::Q6,
        id: "q6",
        label: "Balanced",
        blurb: "Nearly identical to best quality, about 20% smaller.",
        file: "humanizer-12b-Q6_K.gguf",
        size_bytes: 10 * GB,
        min_ram_gb: 16,
    },
    TierInfo {
        tier: Tier::Q4,
        id: "q4",
        label: "Compact",
        blurb: "Slight quality loss. Good for machines under 16 GB.",
        file: "humanizer-12b-Q4_K_M.gguf",
        size_bytes: 76 * GB / 10,
        min_ram_gb: 12,
    },
    TierInfo {
        tier: Tier::Q3,
        id: "q3",
        label: "Small",
        blurb: "Small quality loss, with a few more fact slips.",
        file: "humanizer-12b-Q3-QAT.gguf",
        size_bytes: 56 * GB / 10,
        min_ram_gb: 10,
    },
    TierInfo {
        tier: Tier::Q2,
        id: "q2",
        label: "Smallest",
        blurb: "Runs on 8 GB. Check numbers and names in the result.",
        file: "humanizer-12b-IQ2_XS-QAT.gguf",
        size_bytes: 39 * GB / 10,
        min_ram_gb: 8,
    },
];

const REPO_BASE: &str = "https://huggingface.co/jialinyyzz/humanizer/resolve/main";

impl Tier {
    pub fn info(self) -> &'static TierInfo {
        TIERS
            .iter()
            .find(|info| info.tier == self)
            .expect("every tier is in the catalog")
    }

    pub fn from_id(id: &str) -> Option<Tier> {
        TIERS
            .iter()
            .find(|info| info.id == id)
            .map(|info| info.tier)
    }

    pub fn url(self) -> String {
        format!("{REPO_BASE}/{}", self.info().file)
    }

    /// Folder holding downloaded models, inside Gramflow's config folder.
    pub fn models_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("harper-desktop").join("humanizer").join("models"))
    }

    pub fn path(self) -> Option<PathBuf> {
        Self::models_dir().map(|dir| dir.join(self.info().file))
    }

    /// Partial download kept next to the final file so it can resume after an interruption.
    pub fn partial_path(self) -> Option<PathBuf> {
        self.path().map(|path| path.with_extension("gguf.part"))
    }

    /// A download that is being tried out, or set aside, next to the model in use.
    pub fn update_path(self) -> Option<PathBuf> {
        self.path().map(|path| path.with_extension("gguf.update"))
    }

    /// The model this one replaced in the last update, kept so the update can be undone.
    pub fn previous_path(self) -> Option<PathBuf> {
        self.path().map(|path| path.with_extension("gguf.previous"))
    }

    /// A finished download has at least 99% of the expected size; the catalog sizes are rounded.
    pub fn is_installed(self) -> bool {
        self.path()
            .and_then(|path| std::fs::metadata(path).ok())
            .is_some_and(|meta| meta.len() >= self.info().size_bytes / 100 * 99)
    }
}

/// What we know about a downloaded model beyond its file name.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledModel {
    /// The manifest version this file came from; empty for models downloaded before updates existed.
    pub version: String,
    /// Size in bytes of the file when it was recorded, used to tell if a plain download is current.
    pub size_bytes: u64,
    /// Version to restore when an update is undone.
    pub previous_version: Option<String>,
    pub previous_size_bytes: Option<u64>,
}

pub type InstalledModels = BTreeMap<String, InstalledModel>;

fn installed_records_path() -> Option<PathBuf> {
    Tier::models_dir().map(|dir| dir.join("installed.json"))
}

/// Reads the version records. A missing or damaged file just means "no records".
pub fn read_installed() -> InstalledModels {
    installed_records_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn write_installed(records: &InstalledModels) -> std::io::Result<()> {
    let path = installed_records_path()
        .ok_or_else(|| std::io::Error::other("no place to save model records"))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, serde_json::to_string_pretty(records)?)?;
    std::fs::rename(temporary, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_ordered_largest_first_and_ids_are_unique() {
        let sizes: Vec<_> = TIERS.iter().map(|t| t.size_bytes).collect();
        assert!(sizes.windows(2).all(|pair| pair[0] > pair[1]));
        let rams: Vec<_> = TIERS.iter().map(|t| t.min_ram_gb).collect();
        assert!(rams.windows(2).all(|pair| pair[0] > pair[1]));
        for info in &TIERS {
            assert_eq!(Tier::from_id(info.id), Some(info.tier));
        }
    }

    #[test]
    fn urls_point_at_the_model_repository() {
        assert_eq!(
            Tier::Q8.url(),
            "https://huggingface.co/jialinyyzz/humanizer/resolve/main/humanizer-12b-Q8_0.gguf"
        );
    }

    #[test]
    fn partial_path_sits_beside_the_final_file() {
        let path = Tier::Q6.path().unwrap();
        let partial = Tier::Q6.partial_path().unwrap();
        assert_eq!(path.parent(), partial.parent());
        assert_ne!(path, partial);
    }

    #[test]
    fn update_and_previous_paths_sit_beside_the_model() {
        let path = Tier::Q4.path().unwrap();
        assert_eq!(Tier::Q4.update_path().unwrap().parent(), path.parent());
        assert_eq!(Tier::Q4.previous_path().unwrap().parent(), path.parent());
        assert_ne!(Tier::Q4.update_path(), Tier::Q4.previous_path());
        assert_ne!(Tier::Q4.update_path().unwrap(), path);
    }

    #[test]
    fn records_round_trip_as_json() {
        let mut records = InstalledModels::new();
        records.insert(
            "q8".into(),
            InstalledModel {
                version: "2026-10-15".into(),
                size_bytes: 12_669_630_368,
                previous_version: Some("2026-10-02".into()),
                previous_size_bytes: Some(12_000_000_000),
            },
        );
        let json = serde_json::to_string(&records).unwrap();
        assert_eq!(
            serde_json::from_str::<InstalledModels>(&json).unwrap(),
            records
        );
    }
}
