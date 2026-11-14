//! Tauri commands for Humanizer. The frontend calls these through `Client` in `client.ts`.

use super::capability::{self, Verdict};
use super::download::{self, DownloadError};
use super::engine::{self, Engine, EngineError};
use super::manifest::{self, Manifest, Update};
use super::models::{TIERS, Tier};
use super::numbers::{self, NumberCheck};
use super::settings::HumanizerSettings;
use super::updates;
use crate::config::Config;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};
use tokio::sync::Mutex;

/// Runtime state: the running engine and the cancel switches for downloads and rewrites.
#[derive(Default)]
pub struct HumanizerState {
    engine: Mutex<Option<Engine>>,
    downloading: StdMutex<Option<(Tier, Arc<AtomicBool>)>>,
    rewrite_cancel: StdMutex<Arc<AtomicBool>>,
    /// Text captured by the shortcut before the Humanize window finished loading.
    pending: StdMutex<Option<String>>,
    /// The last approved-model list we fetched, and the updates it offers for installed models.
    manifest: StdMutex<Option<Manifest>>,
    updates: StdMutex<Vec<Update>>,
    update_error: StdMutex<Option<String>>,
}

#[derive(Serialize)]
pub struct Status {
    pub verdict: Verdict,
    pub settings: HumanizerSettings,
    /// The rewriting engine is bundled with this copy of Gramflow.
    pub engine_installed: bool,
    /// Id of the model size being downloaded right now.
    pub downloading: Option<&'static str>,
    /// The engine is loaded in memory.
    pub engine_running: bool,
    /// Id of the model size rewrites will use, when one is installed.
    pub active: Option<&'static str>,
    /// Approved updates for models that are installed.
    pub updates: Vec<Update>,
    /// Why the last update check failed, if it did.
    pub update_error: Option<String>,
    /// Sizes whose last update can be undone.
    pub rollback: Vec<&'static str>,
}

#[derive(Serialize)]
pub struct RewriteResult {
    pub text: String,
    /// Present when number warnings are on.
    pub numbers: Option<NumberCheck>,
}

#[derive(Serialize, Clone)]
struct DownloadDone {
    tier: &'static str,
    error: Option<String>,
}

fn resource_dir<R: Runtime>(app: &AppHandle<R>) -> Option<std::path::PathBuf> {
    app.path().resource_dir().ok()
}

fn parse_tier(id: &str) -> Result<Tier, String> {
    Tier::from_id(id).ok_or_else(|| format!("Unknown model size \"{id}\"."))
}

/// The model rewrites will use: the saved choice if it is installed, else the best installed one.
fn active_tier(settings: &HumanizerSettings) -> Option<Tier> {
    settings
        .active
        .as_deref()
        .and_then(Tier::from_id)
        .filter(|tier| tier.is_installed())
        .or_else(|| {
            TIERS
                .iter()
                .map(|info| info.tier)
                .find(|tier| tier.is_installed())
        })
}

async fn save(config: &Config) -> Result<(), String> {
    config
        .save_to_system()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn humanizer_status<R: Runtime>(
    app: AppHandle<R>,
    config: State<'_, Arc<Mutex<Config>>>,
    state: State<'_, HumanizerState>,
) -> Result<Status, String> {
    let settings = config.lock().await.humanizer.clone();
    let verdict = capability::assess(capability::detect(), Tier::is_installed);
    let engine_running = match state.engine.lock().await.as_mut() {
        Some(engine) => engine.is_running(),
        None => false,
    };
    let downloading = state
        .downloading
        .lock()
        .map_err(|error| error.to_string())?
        .as_ref()
        .map(|(tier, _)| tier.info().id);

    Ok(Status {
        engine_installed: engine::find_binary(resource_dir(&app).as_deref()).is_some(),
        active: active_tier(&settings).map(|tier| tier.info().id),
        verdict,
        settings,
        downloading,
        engine_running,
        updates: state
            .updates
            .lock()
            .map(|list| list.clone())
            .unwrap_or_default(),
        update_error: state
            .update_error
            .lock()
            .ok()
            .and_then(|error| error.clone()),
        rollback: TIERS
            .iter()
            .filter(|info| updates::can_rollback(info.tier))
            .map(|info| info.id)
            .collect(),
    })
}

#[tauri::command]
pub async fn humanizer_save_settings(
    settings: HumanizerSettings,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let mut config = config.lock().await;
    config.humanizer = settings;
    save(&config).await
}

/// Starts a background download. Progress arrives as `humanizer-download` events and the end as
/// one `humanizer-download-done` event.
#[tauri::command]
pub async fn humanizer_download<R: Runtime>(
    tier: String,
    app: AppHandle<R>,
    state: State<'_, HumanizerState>,
) -> Result<(), String> {
    let tier = parse_tier(&tier)?;
    let verdict = capability::assess(capability::detect(), Tier::is_installed);
    let option = verdict
        .options
        .iter()
        .find(|option| option.id == tier.info().id)
        .ok_or("Humanizer isn't available on this computer.")?;
    if !option.fits_memory {
        return Err(format!(
            "This model needs {} GB of memory.",
            tier.info().min_ram_gb
        ));
    }
    if !option.fits_disk && !tier.is_installed() {
        return Err("There isn't enough free disk space for this model.".into());
    }

    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut downloading = state
            .downloading
            .lock()
            .map_err(|error| error.to_string())?;
        if downloading.is_some() {
            return Err("A model is already downloading.".into());
        }
        *downloading = Some((tier, cancel.clone()));
    }

    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let progress_handle = handle.clone();
        let result = download::download(tier, cancel, move |progress| {
            let _ = progress_handle.emit("humanizer-download", progress);
        })
        .await;

        if result.is_ok()
            && let Some(config) = handle.try_state::<Arc<Mutex<Config>>>()
        {
            let mut config = config.lock().await;
            if config.humanizer.active.is_none() {
                config.humanizer.active = Some(tier.info().id.to_string());
            }
            config.humanizer.skipped = false;
            let _ = config.save_to_system().await;
        }
        if let Some(state) = handle.try_state::<HumanizerState>()
            && let Ok(mut downloading) = state.downloading.lock()
        {
            *downloading = None;
        }
        let error = match result {
            Ok(()) | Err(DownloadError::Cancelled) => None,
            Err(error) => Some(error.to_string()),
        };
        let _ = handle.emit(
            "humanizer-download-done",
            DownloadDone {
                tier: tier.info().id,
                error,
            },
        );
    });
    Ok(())
}

#[tauri::command]
pub fn humanizer_cancel_download(state: State<'_, HumanizerState>) -> Result<(), String> {
    if let Some((_, cancel)) = state
        .downloading
        .lock()
        .map_err(|error| error.to_string())?
        .as_ref()
    {
        cancel.store(true, Ordering::Relaxed);
    }
    Ok(())
}

/// Deletes a model's files. If it was the active one, the next best installed model takes over.
#[tauri::command]
pub async fn humanizer_delete_model(
    tier: String,
    config: State<'_, Arc<Mutex<Config>>>,
    state: State<'_, HumanizerState>,
) -> Result<(), String> {
    let tier = parse_tier(&tier)?;
    if state
        .downloading
        .lock()
        .map_err(|error| error.to_string())?
        .as_ref()
        .is_some_and(|(downloading, _)| *downloading == tier)
    {
        return Err("Cancel the download first.".into());
    }

    {
        // Free the model file before removing it; Windows won't delete a file that is in use.
        let mut engine = state.engine.lock().await;
        if engine.as_ref().is_some_and(|engine| engine.tier == tier) {
            *engine = None;
        }
    }

    for path in [tier.path(), tier.partial_path()].into_iter().flatten() {
        match std::fs::remove_file(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("Couldn't delete the model: {error}")),
        }
    }

    let mut config = config.lock().await;
    if config.humanizer.active.as_deref() == Some(tier.info().id) {
        config.humanizer.active = TIERS
            .iter()
            .map(|info| info.tier)
            .find(|candidate| candidate.is_installed())
            .map(|tier| tier.info().id.to_string());
    }
    save(&config).await
}

#[tauri::command]
pub async fn humanizer_set_active_model(
    tier: String,
    config: State<'_, Arc<Mutex<Config>>>,
    state: State<'_, HumanizerState>,
) -> Result<(), String> {
    let tier = parse_tier(&tier)?;
    if !tier.is_installed() {
        return Err("That model isn't downloaded yet.".into());
    }
    // The running engine holds the old model; it reloads with the new one on the next rewrite.
    *state.engine.lock().await = None;

    let mut config = config.lock().await;
    config.humanizer.active = Some(tier.info().id.to_string());
    save(&config).await
}

/// Rewrites `text`. New text streams as `humanizer-text` events, and the call returns the final
/// text plus a number check. The engine starts on first use, which can take several seconds.
#[tauri::command]
pub async fn humanizer_rewrite<R: Runtime>(
    text: String,
    app: AppHandle<R>,
    config: State<'_, Arc<Mutex<Config>>>,
    state: State<'_, HumanizerState>,
) -> Result<RewriteResult, String> {
    let settings = config.lock().await.humanizer.clone();
    let tier = active_tier(&settings).ok_or("Set up Humanizer first.")?;

    let cancel = Arc::new(AtomicBool::new(false));
    *state
        .rewrite_cancel
        .lock()
        .map_err(|error| error.to_string())? = cancel.clone();

    let mut guard = state.engine.lock().await;
    let needs_start = match guard.as_mut() {
        Some(engine) => engine.tier != tier || !engine.is_running(),
        None => true,
    };
    if needs_start {
        *guard = None;
        let binary = engine::find_binary(resource_dir(&app).as_deref())
            .ok_or_else(|| EngineError::NoBinary.to_string())?;
        *guard = Some(
            Engine::start(&binary, tier)
                .await
                .map_err(|error| error.to_string())?,
        );
    }
    let engine = guard
        .as_mut()
        .ok_or("The rewriting engine isn't running.")?;

    let emitter = app.clone();
    let rewritten = engine
        .rewrite(&text, cancel, move |piece| {
            let _ = emitter.emit("humanizer-text", piece);
        })
        .await
        .map_err(|error| error.to_string())?;

    let numbers = settings
        .warn_numbers
        .then(|| numbers::check(&text, &rewritten));
    Ok(RewriteResult {
        text: rewritten,
        numbers,
    })
}

#[tauri::command]
pub fn humanizer_cancel_rewrite(state: State<'_, HumanizerState>) -> Result<(), String> {
    state
        .rewrite_cancel
        .lock()
        .map_err(|error| error.to_string())?
        .store(true, Ordering::Relaxed);
    Ok(())
}

#[tauri::command]
pub async fn humanizer_stop_engine(state: State<'_, HumanizerState>) -> Result<(), String> {
    *state.engine.lock().await = None;
    Ok(())
}

#[tauri::command]
pub fn humanizer_open_models_folder<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;

    let dir = Tier::models_dir().ok_or("Couldn't find the models folder.")?;
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(|error| error.to_string())
}

/// Unloads the engine after the configured idle time, so the model stops using memory.
pub fn spawn_idle_watcher<R: Runtime>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            let (Some(config), Some(state)) = (
                app.try_state::<Arc<Mutex<Config>>>(),
                app.try_state::<HumanizerState>(),
            ) else {
                continue;
            };
            let minutes = config.lock().await.humanizer.idle_minutes;
            if minutes == 0 {
                continue;
            }
            let mut engine = state.engine.lock().await;
            if engine
                .as_ref()
                .is_some_and(|engine| engine.idle_for() >= Duration::from_secs(minutes * 60))
            {
                *engine = None;
            }
        }
    });
}

/// Text captured by the shortcut, handed over once to the Humanize window when it loads.
#[tauri::command]
pub fn humanizer_take_pending(state: State<'_, HumanizerState>) -> Option<String> {
    state
        .pending
        .lock()
        .ok()
        .and_then(|mut pending| pending.take())
}

/// Asks the main process to open Settings; it owns the windows (see `main.rs`).
#[tauri::command]
pub fn humanizer_open_settings<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    app.emit("humanizer-open-settings", ())
        .map_err(|error| error.to_string())
}

/// Asks the main process to open the Humanize window.
#[tauri::command]
pub fn humanizer_open_window<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    app.emit("humanizer-open-window", ())
        .map_err(|error| error.to_string())
}

/// Hides the Humanize window, then pastes `text` over the selection in the app underneath.
#[tauri::command]
pub async fn humanizer_paste_back<R: Runtime>(
    text: String,
    app: AppHandle<R>,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("editor") {
        window.hide().map_err(|error| error.to_string())?;
    }
    #[cfg(target_os = "macos")]
    let _ = app.hide();
    // Give the previous app a moment to take focus back before pasting.
    tokio::time::sleep(Duration::from_millis(350)).await;
    tokio::task::spawn_blocking(move || super::selection::paste_text(&text))
        .await
        .map_err(|error| error.to_string())?
}

/// Runs when the user presses the Humanize shortcut: copies the selection, opens the window,
/// and hands it the text. With nothing selected the window opens empty.
pub fn handle_shortcut(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let captured = tokio::task::spawn_blocking(super::selection::capture_selection)
            .await
            .ok()
            .and_then(|result| result.ok())
            .flatten();

        if let Some(state) = app.try_state::<HumanizerState>()
            && let Ok(mut pending) = state.pending.lock()
        {
            *pending = captured.clone();
        }
        let _ = crate::windows::show_humanize_window(&app);
        let _ = app.emit("humanize-input", captured);
    });
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0)
}

fn installed_file_size(tier: Tier) -> Option<u64> {
    tier.path()
        .filter(|_| tier.is_installed())
        .and_then(|path| std::fs::metadata(path).ok())
        .map(|meta| meta.len())
}

/// Fetches the approved-model list and works out which installed models it would update.
async fn run_update_check<R: Runtime>(app: &AppHandle<R>) -> Result<Vec<Update>, String> {
    let result = async {
        let manifest = updates::fetch_manifest().await?;
        let version = app.package_info().version.to_string();
        let list = manifest::available_updates(
            &manifest,
            &version,
            &super::models::read_installed(),
            installed_file_size,
        );
        Ok::<_, String>((manifest, list))
    }
    .await;

    if let Some(state) = app.try_state::<HumanizerState>() {
        match &result {
            Ok((manifest, list)) => {
                if let Ok(mut slot) = state.manifest.lock() {
                    *slot = Some(manifest.clone());
                }
                if let Ok(mut slot) = state.updates.lock() {
                    *slot = list.clone();
                }
                if let Ok(mut slot) = state.update_error.lock() {
                    *slot = None;
                }
            }
            Err(error) => {
                if let Ok(mut slot) = state.update_error.lock() {
                    *slot = Some(error.clone());
                }
            }
        }
    }
    // A failed check still counts, so a broken connection isn't retried every minute.
    if let Some(config) = app.try_state::<Arc<Mutex<Config>>>() {
        let mut config = config.lock().await;
        config.humanizer.last_update_check = Some(unix_now());
        let _ = config.save_to_system().await;
    }
    result.map(|(_, list)| list)
}

/// Looks for approved model updates now. Updates are only ever offered, never installed here.
#[tauri::command]
pub async fn humanizer_check_updates<R: Runtime>(app: AppHandle<R>) -> Result<Vec<Update>, String> {
    run_update_check(&app).await
}

#[derive(Serialize, Clone)]
struct UpdateDone {
    tier: &'static str,
    error: Option<String>,
}

#[derive(Serialize, Clone)]
struct UpdatePhase {
    tier: &'static str,
    phase: &'static str,
}

/// Downloads, tests, and installs one approved update in the background. Progress arrives as
/// `humanizer-download` and `humanizer-update` events, the end as `humanizer-update-done`.
#[tauri::command]
pub async fn humanizer_apply_update<R: Runtime>(
    tier: String,
    app: AppHandle<R>,
    state: State<'_, HumanizerState>,
) -> Result<(), String> {
    let tier = parse_tier(&tier)?;
    let id = tier.info().id;
    let entry = {
        let manifest = state.manifest.lock().map_err(|error| error.to_string())?;
        let offered = state
            .updates
            .lock()
            .map_err(|error| error.to_string())?
            .iter()
            .find(|update| update.tier == id)
            .cloned()
            .ok_or("There's no update for this model. Check for updates first.")?;
        if offered.needs_newer_app {
            return Err(
                "Update Gramflow first. This model needs a newer version of the app.".into(),
            );
        }
        manifest
            .as_ref()
            .and_then(|manifest| manifest.models.iter().find(|model| model.tier == id))
            .cloned()
            .ok_or("Check for updates first.")?
    };
    let binary = engine::find_binary(resource_dir(&app).as_deref())
        .ok_or_else(|| EngineError::NoBinary.to_string())?;

    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut downloading = state
            .downloading
            .lock()
            .map_err(|error| error.to_string())?;
        if downloading.is_some() {
            return Err("A model is already downloading.".into());
        }
        *downloading = Some((tier, cancel.clone()));
    }

    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let free_disk_gb = capability::detect().free_disk_gb;
        let phase_handle = handle.clone();
        let progress_handle = handle.clone();
        let prepared = updates::prepare(
            tier,
            &entry,
            &binary,
            free_disk_gb,
            cancel,
            move |phase| {
                let name = match phase {
                    updates::Phase::Downloading => "downloading",
                    updates::Phase::Testing => "testing",
                };
                let _ = phase_handle.emit(
                    "humanizer-update",
                    UpdatePhase {
                        tier: id,
                        phase: name,
                    },
                );
            },
            move |progress| {
                let _ = progress_handle.emit("humanizer-download", progress);
            },
        )
        .await;

        let result = match prepared {
            Ok(()) => {
                let _ = handle.emit(
                    "humanizer-update",
                    UpdatePhase {
                        tier: id,
                        phase: "installing",
                    },
                );
                if let Some(state) = handle.try_state::<HumanizerState>() {
                    // Free the old model file before swapping it; Windows can't move a file in use.
                    let mut engine = state.engine.lock().await;
                    if engine.as_ref().is_some_and(|engine| engine.tier == tier) {
                        *engine = None;
                    }
                }
                updates::install(tier, &entry)
            }
            Err(error) => Err(error),
        };

        if result.is_ok()
            && let Some(state) = handle.try_state::<HumanizerState>()
            && let Ok(mut list) = state.updates.lock()
        {
            list.retain(|update| update.tier != id);
        }
        if let Some(state) = handle.try_state::<HumanizerState>()
            && let Ok(mut downloading) = state.downloading.lock()
        {
            *downloading = None;
        }
        let error = match result {
            Ok(()) | Err(updates::UpdateError::Cancelled) => None,
            Err(error) => Some(error.to_string()),
        };
        let _ = handle.emit("humanizer-update-done", UpdateDone { tier: id, error });
    });
    Ok(())
}

/// Goes back to the model an update replaced. Running it again returns to the updated one.
#[tauri::command]
pub async fn humanizer_rollback(
    tier: String,
    state: State<'_, HumanizerState>,
) -> Result<(), String> {
    let tier = parse_tier(&tier)?;
    if state
        .downloading
        .lock()
        .map_err(|error| error.to_string())?
        .is_some()
    {
        return Err("Wait for the download to finish first.".into());
    }
    {
        let mut engine = state.engine.lock().await;
        if engine.as_ref().is_some_and(|engine| engine.tier == tier) {
            *engine = None;
        }
    }
    updates::rollback(tier).map_err(|error| error.to_string())?;
    // The list of offered updates is out of date now; the next check rebuilds it.
    if let Ok(mut list) = state.updates.lock() {
        list.clear();
    }
    Ok(())
}

/// Checks for approved model updates about once a day, if the user hasn't turned that off. It
/// only looks; installing an update always needs a click.
pub fn spawn_update_checker<R: Runtime>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(90)).await;
        loop {
            let due = match app.try_state::<Arc<Mutex<Config>>>() {
                Some(config) => {
                    let humanizer = config.lock().await.humanizer.clone();
                    humanizer.auto_check_updates
                        && unix_now().saturating_sub(humanizer.last_update_check.unwrap_or(0))
                            >= 24 * 60 * 60
                }
                None => false,
            };
            if due && TIERS.iter().any(|info| info.tier.is_installed()) {
                let _ = run_update_check(&app).await;
            }
            tokio::time::sleep(Duration::from_secs(60 * 60)).await;
        }
    });
}
