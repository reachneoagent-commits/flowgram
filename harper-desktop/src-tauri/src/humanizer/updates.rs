//! Fetching the approved-model list and applying an update safely.
//!
//! An update never touches the model in use until the new file has been downloaded, matched
//! against its checksum, and shown to produce a sane rewrite. The old model is then kept beside
//! it so the update can be undone.

use super::download::{self, DownloadError, Progress};
use super::engine::Engine;
use super::manifest::{self, MANIFEST_URL, Manifest, ManifestModel};
use super::models::{InstalledModel, InstalledModels, Tier};
use super::numbers;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

/// Free disk space to keep beyond the download itself.
const DISK_HEADROOM_GB: f64 = 1.0;

/// A fixed draft with facts to preserve, used to try a new model before it is adopted.
pub const SMOKE_DRAFT: &str = "Thanks for reaching out. I wanted to circle back and leverage our previous discussion to ensure alignment. The budget is $180K and the deadline is March 3.";

pub async fn fetch_manifest() -> Result<Manifest, String> {
    let response = reqwest::Client::builder()
        .user_agent("Gramflow")
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|error| error.to_string())?
        .get(MANIFEST_URL)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|error| format!("Couldn't reach the update list: {error}"))?;
    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("Couldn't read the update list: {error}"))?;
    let text =
        std::str::from_utf8(&bytes).map_err(|_| "The update list isn't text.".to_string())?;
    manifest::parse(text).map_err(|error| error.to_string())
}

/// Decides whether a try-out rewrite looks sane: some text, not wildly longer than the draft,
/// not drifted into another script, and with every number intact.
pub fn smoke_check(draft: &str, rewrite: &str) -> Result<(), String> {
    let rewrite = rewrite.trim();
    if rewrite.is_empty() {
        return Err("the new model returned nothing".into());
    }
    if rewrite.chars().count() > draft.chars().count() * 3 {
        return Err("the new model's rewrite was far longer than the draft".into());
    }
    if rewrite
        .chars()
        .any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c))
    {
        return Err("the new model answered in the wrong language".into());
    }
    let numbers = numbers::check(draft, rewrite);
    if !numbers.is_clean() {
        return Err("the new model changed a number in the test text".into());
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("There isn't enough free disk space for this update.")]
    NoSpace,
    #[error("{0}")]
    Download(#[from] DownloadError),
    #[error("The new model didn't start: {0}")]
    Engine(String),
    #[error("The new model didn't pass its test: {0}")]
    Smoke(String),
    #[error("Couldn't install the update: {0}")]
    Files(#[from] std::io::Error),
    #[error("The update was cancelled.")]
    Cancelled,
}

/// Which step an update is on, reported to the interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Downloading,
    Testing,
}

/// Downloads the approved model next to the one in use and proves it works. On any failure the
/// downloaded file is removed and the current model is left exactly as it was.
pub async fn prepare(
    tier: Tier,
    entry: &ManifestModel,
    engine_binary: &Path,
    free_disk_gb: f64,
    cancel: Arc<AtomicBool>,
    mut on_phase: impl FnMut(Phase),
    on_progress: impl FnMut(Progress),
) -> Result<(), UpdateError> {
    if free_disk_gb < entry.size_bytes as f64 / 1e9 + DISK_HEADROOM_GB {
        return Err(UpdateError::NoSpace);
    }
    let update_path = tier.update_path().ok_or(DownloadError::NoFolder)?;
    let part_path = update_path.with_extension("update.part");

    on_phase(Phase::Downloading);
    let downloaded = download::download_file(
        tier.info().id,
        &entry.url,
        &part_path,
        &update_path,
        Some(&entry.sha256),
        cancel,
        on_progress,
    )
    .await;
    match downloaded {
        Ok(()) => {}
        Err(DownloadError::Cancelled) => return Err(UpdateError::Cancelled),
        Err(error) => {
            let _ = std::fs::remove_file(&part_path);
            return Err(error.into());
        }
    }

    on_phase(Phase::Testing);
    let tried = try_model(engine_binary, &update_path, tier).await;
    // Let the engine release the file before anything else touches it (Windows holds it open).
    tokio::time::sleep(Duration::from_millis(600)).await;
    if let Err(error) = tried {
        let _ = std::fs::remove_file(&update_path);
        return Err(error);
    }
    Ok(())
}

async fn try_model(binary: &Path, model: &Path, tier: Tier) -> Result<(), UpdateError> {
    let mut engine = Engine::start_with_model(binary, model, tier)
        .await
        .map_err(|error| UpdateError::Engine(error.to_string()))?;
    let rewrite = engine
        .rewrite(SMOKE_DRAFT, Arc::new(AtomicBool::new(false)), |_| {})
        .await
        .map_err(|error| UpdateError::Engine(error.to_string()))?;
    smoke_check(SMOKE_DRAFT, &rewrite).map_err(UpdateError::Smoke)
}

/// Puts the tried model in use and keeps the old one as the way back. The engine must not be
/// running on this model.
pub fn install(tier: Tier, entry: &ManifestModel) -> Result<(), UpdateError> {
    let path = tier.path().ok_or(DownloadError::NoFolder)?;
    let update = tier.update_path().ok_or(DownloadError::NoFolder)?;
    let previous = tier.previous_path().ok_or(DownloadError::NoFolder)?;
    install_files(&path, &update, &previous)?;

    let mut records = super::models::read_installed();
    let next = record_after_update(
        records.get(tier.info().id),
        &entry.version,
        entry.size_bytes,
    );
    records.insert(tier.info().id.to_string(), next);
    super::models::write_installed(&records)?;
    Ok(())
}

/// Swaps the model back to the one the last update replaced. Undoing again re-applies the update.
pub fn rollback(tier: Tier) -> Result<(), UpdateError> {
    let path = tier.path().ok_or(DownloadError::NoFolder)?;
    let previous = tier.previous_path().ok_or(DownloadError::NoFolder)?;
    let scratch = tier.update_path().ok_or(DownloadError::NoFolder)?;
    swap_files(&path, &previous, &scratch)?;

    let mut records = super::models::read_installed();
    if let Some(record) = records.get_mut(tier.info().id) {
        *record = record_after_rollback(record);
    }
    super::models::write_installed(&records)?;
    Ok(())
}

pub fn can_rollback(tier: Tier) -> bool {
    tier.previous_path().is_some_and(|path| path.is_file())
}

fn install_files(path: &Path, update: &Path, previous: &Path) -> std::io::Result<()> {
    if path.is_file() {
        let _ = std::fs::remove_file(previous);
        std::fs::rename(path, previous)?;
    }
    if let Err(error) = std::fs::rename(update, path) {
        // Put the old model back so a failed install leaves things as they were.
        if previous.is_file() {
            let _ = std::fs::rename(previous, path);
        }
        return Err(error);
    }
    Ok(())
}

fn swap_files(path: &Path, previous: &Path, scratch: &Path) -> std::io::Result<()> {
    if !previous.is_file() {
        return Err(std::io::Error::other(
            "there is no earlier model to go back to",
        ));
    }
    let _ = std::fs::remove_file(scratch);
    std::fs::rename(path, scratch)?;
    if let Err(error) = std::fs::rename(previous, path) {
        let _ = std::fs::rename(scratch, path);
        return Err(error);
    }
    std::fs::rename(scratch, previous)
}

fn record_after_update(
    old: Option<&InstalledModel>,
    version: &str,
    size_bytes: u64,
) -> InstalledModel {
    InstalledModel {
        version: version.to_string(),
        size_bytes,
        previous_version: Some(
            old.map(|old| old.version.clone())
                .filter(|version| !version.is_empty())
                .unwrap_or_else(|| "original".into()),
        ),
        previous_size_bytes: old.map(|old| old.size_bytes),
    }
}

fn record_after_rollback(record: &InstalledModel) -> InstalledModel {
    InstalledModel {
        version: record.previous_version.clone().unwrap_or_default(),
        size_bytes: record.previous_size_bytes.unwrap_or(0),
        previous_version: Some(record.version.clone()),
        previous_size_bytes: Some(record.size_bytes),
    }
}

/// Records the version of a model that was just downloaded fresh from the approved list.
pub fn record_fresh(tier: Tier, version: &str, size_bytes: u64) {
    let mut records: InstalledModels = super::models::read_installed();
    records.insert(
        tier.info().id.to_string(),
        InstalledModel {
            version: version.to_string(),
            size_bytes,
            ..Default::default()
        },
    );
    let _ = super::models::write_installed(&records);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("gramflow-updates-{name}-{}", fastrand::u32(..)));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(path: &Path, text: &str) {
        std::fs::write(path, text).unwrap();
    }

    fn read(path: &Path) -> String {
        std::fs::read_to_string(path).unwrap()
    }

    #[test]
    fn a_good_rewrite_passes_the_smoke_check() {
        assert!(
            smoke_check(
                SMOKE_DRAFT,
                "Just following up. The budget is $180K and the deadline is March 3."
            )
            .is_ok()
        );
    }

    #[test]
    fn the_smoke_check_rejects_bad_rewrites() {
        let cases = [
            ("empty", ""),
            ("blank", "   "),
            (
                "changed number",
                "Just following up. The budget is $180K and the deadline is March 5.",
            ),
            ("dropped number", "Just following up. The budget is $180K."),
            ("far too long", &"word ".repeat(200)),
            (
                "wrong language",
                "只是跟进一下。预算是$180K,截止日期是3月3日。",
            ),
        ];
        for (name, rewrite) in cases {
            assert!(
                smoke_check(SMOKE_DRAFT, rewrite).is_err(),
                "{name} should fail"
            );
        }
    }

    #[test]
    fn installing_keeps_the_old_model_as_previous() {
        let dir = temp("install");
        let (path, update, previous) = (
            dir.join("m.gguf"),
            dir.join("m.gguf.update"),
            dir.join("m.gguf.previous"),
        );
        write(&path, "old");
        write(&update, "new");
        install_files(&path, &update, &previous).unwrap();
        assert_eq!(read(&path), "new");
        assert_eq!(read(&previous), "old");
        assert!(!update.exists());
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn a_second_update_replaces_the_older_backup() {
        let dir = temp("second");
        let (path, update, previous) = (
            dir.join("m.gguf"),
            dir.join("m.gguf.update"),
            dir.join("m.gguf.previous"),
        );
        write(&path, "v2");
        write(&previous, "v1");
        write(&update, "v3");
        install_files(&path, &update, &previous).unwrap();
        assert_eq!(read(&path), "v3");
        assert_eq!(read(&previous), "v2");
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn a_failed_install_leaves_the_old_model_in_place() {
        let dir = temp("failed");
        let (path, update, previous) = (
            dir.join("m.gguf"),
            dir.join("missing.update"),
            dir.join("m.gguf.previous"),
        );
        write(&path, "old");
        assert!(install_files(&path, &update, &previous).is_err());
        assert_eq!(read(&path), "old");
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn rolling_back_swaps_and_can_be_undone() {
        let dir = temp("rollback");
        let (path, previous, scratch) = (
            dir.join("m.gguf"),
            dir.join("m.gguf.previous"),
            dir.join("m.gguf.update"),
        );
        write(&path, "new");
        write(&previous, "old");
        swap_files(&path, &previous, &scratch).unwrap();
        assert_eq!((read(&path), read(&previous)), ("old".into(), "new".into()));
        swap_files(&path, &previous, &scratch).unwrap();
        assert_eq!((read(&path), read(&previous)), ("new".into(), "old".into()));
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn rolling_back_without_a_previous_model_fails_cleanly() {
        let dir = temp("nothing");
        let path = dir.join("m.gguf");
        write(&path, "only");
        assert!(swap_files(&path, &dir.join("none"), &dir.join("scratch")).is_err());
        assert_eq!(read(&path), "only");
        std::fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn records_track_the_version_chain() {
        let original = InstalledModel {
            version: String::new(),
            size_bytes: 10,
            ..Default::default()
        };
        let updated = record_after_update(Some(&original), "v2", 20);
        assert_eq!((updated.version.as_str(), updated.size_bytes), ("v2", 20));
        assert_eq!(updated.previous_version.as_deref(), Some("original"));
        let back = record_after_rollback(&updated);
        assert_eq!((back.version.as_str(), back.size_bytes), ("original", 10));
        assert_eq!(back.previous_version.as_deref(), Some("v2"));
        let forward = record_after_rollback(&back);
        assert_eq!((forward.version.as_str(), forward.size_bytes), ("v2", 20));
    }
}
