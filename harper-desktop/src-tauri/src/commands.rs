//! This module centralizes all the Rust commands to be made available to the JavaScript
//! environment. Use [`application_message_handler`] to load them into the Tauri runtime.

use crate::config::Config;
use crate::desktop_updater::{DesktopUpdater, UpdateResult};
use crate::highlighter_service::HighlighterService;
use crate::os_broker::{AccessibilityPermissionStatus, AppSearchResult, OsBroker};
use crate::{IntegrationView, PlatformBroker};
use base64::{Engine as _, engine::general_purpose};
use harper_core::{
    Dialect, DictWordMetadata, IgnoredLints,
    linting::FlatConfig,
    spell::{Dictionary, MutableDictionary},
};
use std::sync::{Arc, RwLock};
use tauri::ipc::Invoke;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};
use tokio::sync::Mutex;

pub fn application_message_handler<R: Runtime>() -> impl Fn(Invoke<R>) -> bool {
    tauri::generate_handler![
        get_lint_config,
        get_dialect,
        get_debounce_ms,
        set_debounce_ms,
        get_auto_update,
        set_auto_update,
        get_current_version,
        get_latest_version,
        update_to_latest,
        get_onboarding_completed,
        set_onboarding_completed,
        set_dialect,
        set_lint_config,
        get_dictionary,
        set_dictionary,
        ignore_lint,
        add_to_dictionary,
        get_integrations,
        get_auto_enable_new_apps,
        set_auto_enable_new_apps,
        add_integration,
        remove_integration,
        set_integration_enabled,
        get_application_icon_data_url,
        get_accessibility_permission_status,
        request_accessibility_permission,
        start_highlighter_service,
        stop_highlighter_service,
        launch_app,
        prepare_browser_extension,
        prepare_firefox_extension,
        open_browser_extensions_page,
        get_browser_extension_profiles,
        search_apps,
        get_theme,
        set_theme,
        crate::humanizer::commands::humanizer_status,
        crate::humanizer::commands::humanizer_save_settings,
        crate::humanizer::commands::humanizer_download,
        crate::humanizer::commands::humanizer_cancel_download,
        crate::humanizer::commands::humanizer_delete_model,
        crate::humanizer::commands::humanizer_set_active_model,
        crate::humanizer::commands::humanizer_rewrite,
        crate::humanizer::commands::humanizer_cancel_rewrite,
        crate::humanizer::commands::humanizer_stop_engine,
        crate::humanizer::commands::humanizer_open_models_folder,
        crate::humanizer::commands::humanizer_check_updates,
        crate::humanizer::commands::humanizer_apply_update,
        crate::humanizer::commands::humanizer_rollback,
        crate::humanizer::commands::humanizer_take_pending,
        crate::humanizer::commands::humanizer_open_settings,
        crate::humanizer::commands::humanizer_open_window,
        crate::humanizer::commands::humanizer_paste_back,
    ]
}

#[tauri::command]
async fn get_lint_config(config: State<'_, Arc<Mutex<Config>>>) -> Result<FlatConfig, String> {
    let mut lint_config = config.lock().await.lint_config.clone();
    lint_config.fill_with_curated();

    Ok(lint_config)
}

#[tauri::command]
async fn get_dialect(config: State<'_, Arc<Mutex<Config>>>) -> Result<Dialect, String> {
    Ok(config.lock().await.dialect)
}

#[tauri::command]
async fn get_debounce_ms(config: State<'_, Arc<Mutex<Config>>>) -> Result<u64, String> {
    Ok(config.lock().await.debounce_ms)
}

#[tauri::command]
async fn set_debounce_ms(
    debounce_ms: u64,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let mut config = config.lock().await;
    config.debounce_ms = debounce_ms;
    config
        .save_to_system()
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
async fn get_auto_update(config: State<'_, Arc<Mutex<Config>>>) -> Result<bool, String> {
    Ok(config.lock().await.auto_update)
}

#[tauri::command]
async fn set_auto_update(
    auto_update: bool,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let mut config = config.lock().await;
    config.auto_update = auto_update;
    config
        .save_to_system()
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
fn get_current_version<R: Runtime>(app: tauri::AppHandle<R>) -> String {
    DesktopUpdater::current_version(&app)
}

#[tauri::command]
async fn get_latest_version() -> Result<String, String> {
    DesktopUpdater::latest_version().await
}

#[tauri::command]
async fn update_to_latest<R: Runtime>(
    app: tauri::AppHandle<R>,
    updater: State<'_, DesktopUpdater>,
) -> Result<UpdateResult, String> {
    updater
        .update_to_latest(&app, false)
        .await
        .ok_or_else(|| "Manual update check was unexpectedly skipped.".into())
}

#[tauri::command]
async fn get_onboarding_completed(config: State<'_, Arc<Mutex<Config>>>) -> Result<bool, String> {
    Ok(config.lock().await.onboarding_completed)
}

#[tauri::command]
async fn set_onboarding_completed<R: Runtime>(
    onboarding_completed: bool,
    app: AppHandle<R>,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let mut config = config.lock().await;
    let previous_value = config.onboarding_completed;
    config.onboarding_completed = onboarding_completed;

    if let Err(error) = config.save_to_system().await {
        config.onboarding_completed = previous_value;
        return Err(error.to_string());
    }

    // Setup is over: the settings window grows to its roomy size (handled in the main process).
    let _ = app.emit("onboarding-completed", ());

    Ok(())
}

#[tauri::command]
async fn set_dialect(
    dialect: Dialect,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let mut config = config.lock().await;
    config.dialect = dialect;
    config
        .save_to_system()
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
async fn get_theme(config: State<'_, Arc<Mutex<Config>>>) -> Result<String, String> {
    Ok(config.lock().await.theme.clone())
}

#[tauri::command]
async fn set_theme<R: Runtime>(
    theme: String,
    app: AppHandle<R>,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let mut config = config.lock().await;
    config.theme = theme;
    config
        .save_to_system()
        .await
        .map_err(|error| error.to_string())?;

    // Windows pick up the new frame theme in the main process, which owns them.
    let _ = app.emit("app-theme-changed", config.theme.clone());

    Ok(())
}

#[tauri::command]
async fn set_lint_config(
    lint_config: FlatConfig,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let mut lint_config = lint_config;
    lint_config.fill_with_curated();

    let mut config = config.lock().await;
    config.lint_config = lint_config;
    config
        .save_to_system()
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
async fn get_dictionary(config: State<'_, Arc<Mutex<Config>>>) -> Result<Vec<String>, String> {
    let mut words = config
        .lock()
        .await
        .mutable_dictionary
        .words_iter()
        .map(|word| word.iter().collect::<String>())
        .collect::<Vec<_>>();
    words.sort();

    Ok(words)
}

#[tauri::command]
async fn set_dictionary(
    words: Vec<String>,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let mut dictionary = MutableDictionary::new();
    dictionary.extend_words(words.into_iter().map(|word| {
        (
            word.chars().collect::<Vec<_>>(),
            DictWordMetadata::default(),
        )
    }));

    let mut config = config.lock().await;
    config.mutable_dictionary = dictionary;
    config
        .save_to_system()
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
async fn ignore_lint(
    ignored_lints: String,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let ignored_lints =
        serde_json::from_str::<IgnoredLints>(&ignored_lints).map_err(|error| error.to_string())?;

    let mut config = config.lock().await;
    config.ignored_lints.append(ignored_lints);
    config
        .save_to_system()
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
async fn add_to_dictionary(
    word: String,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let mut config = config.lock().await;
    config
        .mutable_dictionary
        .append_word_str(&word, DictWordMetadata::default());
    config
        .save_to_system()
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
async fn get_integrations(
    config: State<'_, Arc<Mutex<Config>>>,
    broker: State<'_, RwLock<PlatformBroker>>,
) -> Result<Vec<IntegrationView>, String> {
    let integrations = config.lock().await.integrations.clone();
    let broker = broker
        .read()
        .map_err(|error| format!("Failed to read platform broker: {error}"))?;

    Ok(integrations
        .into_iter()
        .map(|integration| IntegrationView {
            display_name: broker.integration_display_name(&integration.bundle_id),
            bundle_id: integration.bundle_id,
            enabled: integration.enabled,
        })
        .collect())
}

#[tauri::command]
async fn get_auto_enable_new_apps(config: State<'_, Arc<Mutex<Config>>>) -> Result<bool, String> {
    Ok(config.lock().await.auto_enable_new_apps)
}

#[tauri::command]
async fn set_auto_enable_new_apps(
    auto_enable_new_apps: bool,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let mut config = config.lock().await;
    let previous = config.auto_enable_new_apps;
    config.auto_enable_new_apps = auto_enable_new_apps;
    if let Err(error) = config.save_to_system().await {
        config.auto_enable_new_apps = previous;
        return Err(error.to_string());
    }
    Ok(())
}

#[tauri::command]
async fn add_integration(
    bundle_id: String,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let mut config = config.lock().await;
    config.add_integration(bundle_id);
    config
        .save_to_system()
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
async fn remove_integration(
    bundle_id: String,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let mut config = config.lock().await;
    config.remove_integration(&bundle_id);
    config
        .save_to_system()
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
async fn set_integration_enabled(
    bundle_id: String,
    enabled: bool,
    config: State<'_, Arc<Mutex<Config>>>,
) -> Result<(), String> {
    let mut config = config.lock().await;
    config.set_integration_enabled(&bundle_id, enabled);
    config
        .save_to_system()
        .await
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
async fn get_application_icon_data_url<R: Runtime>(
    bundle_id: String,
    app_handle: tauri::AppHandle<R>,
) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let broker = app_handle.state::<RwLock<PlatformBroker>>();
        let icon_png = broker
            .read()
            .map_err(|error| format!("Failed to read platform broker: {error}"))?
            .application_icon_png(&bundle_id)?;
        let encoded = general_purpose::STANDARD.encode(icon_png);

        Ok(format!("data:image/png;base64,{encoded}"))
    })
    .await
    .map_err(|error| format!("Failed to load application icon: {error}"))?
}

#[tauri::command]
fn get_accessibility_permission_status(
    broker: State<'_, RwLock<PlatformBroker>>,
) -> AccessibilityPermissionStatus {
    match broker.read() {
        Ok(broker) => broker.accessibility_permission_status(),
        Err(error) => {
            eprintln!("Failed to read platform broker: {error}");
            AccessibilityPermissionStatus::Unsupported
        }
    }
}

#[tauri::command]
fn request_accessibility_permission(
    broker: State<'_, RwLock<PlatformBroker>>,
) -> AccessibilityPermissionStatus {
    match broker.read() {
        Ok(broker) => broker.request_accessibility_permission(),
        Err(error) => {
            eprintln!("Failed to read platform broker: {error}");
            AccessibilityPermissionStatus::Unsupported
        }
    }
}

#[tauri::command]
pub(crate) async fn start_highlighter_service(
    config: State<'_, Arc<Mutex<Config>>>,
    highlighter_service: State<'_, HighlighterService>,
) -> Result<bool, String> {
    {
        let mut config = config.lock().await;
        config.highlighter_service_enabled = true;
        config
            .save_to_system()
            .await
            .map_err(|error| error.to_string())?;
    }

    highlighter_service
        .start()
        .map_err(|error| error.to_string())?;

    Ok(highlighter_service.is_running())
}

#[tauri::command]
pub(crate) async fn stop_highlighter_service(
    config: State<'_, Arc<Mutex<Config>>>,
    highlighter_service: State<'_, HighlighterService>,
) -> Result<bool, String> {
    {
        let mut config = config.lock().await;
        config.highlighter_service_enabled = false;
        config
            .save_to_system()
            .await
            .map_err(|error| error.to_string())?;
    }

    Ok(highlighter_service.stop())
}

#[tauri::command]
fn launch_app(bundle_id: String, broker: State<'_, RwLock<PlatformBroker>>) -> Result<(), String> {
    broker
        .read()
        .map_err(|error| format!("Failed to read platform broker: {error}"))?
        .launch_app_bundle(&bundle_id)
}

#[tauri::command]
fn search_apps(
    query: String,
    broker: State<'_, RwLock<PlatformBroker>>,
) -> Result<Vec<AppSearchResult>, String> {
    broker
        .read()
        .map_err(|error| format!("Failed to read platform broker: {error}"))?
        .search_apps(&query)
}

/// Copy the bundled browser extension to a stable, visible folder and open it in Finder.
///
/// Chrome loads unpacked extensions from a fixed path, so the copy must live outside the app bundle
/// and survive app updates. Returns the folder path so the UI can show it.
#[tauri::command]
fn prepare_browser_extension<R: Runtime>(
    app: AppHandle<R>,
    reveal: Option<bool>,
) -> Result<String, String> {
    prepare_extension_folder(
        &app,
        "browser-extension/chrome",
        crate::browser_extension::EXTENSION_FOLDER_NAME,
        reveal.unwrap_or(true),
    )
}

/// Same as [`prepare_browser_extension`], for the Firefox build of the extension.
#[tauri::command]
fn prepare_firefox_extension<R: Runtime>(app: AppHandle<R>) -> Result<String, String> {
    prepare_extension_folder(
        &app,
        "browser-extension/firefox",
        crate::browser_extension::FIREFOX_EXTENSION_FOLDER_NAME,
        true,
    )
}

fn prepare_extension_folder<R: Runtime>(
    app: &AppHandle<R>,
    resource: &str,
    folder_name: &str,
    reveal: bool,
) -> Result<String, String> {
    let source = app
        .path()
        .resolve(resource, BaseDirectory::Resource)
        .map_err(|error| format!("Could not find the bundled extension: {error}"))?;
    let destination = app
        .path()
        .resolve(folder_name, BaseDirectory::Document)
        .map_err(|error| format!("Could not find your Documents folder: {error}"))?;

    if destination.exists() {
        std::fs::remove_dir_all(&destination)
            .map_err(|error| format!("Could not refresh the extension folder: {error}"))?;
    }
    copy_dir_recursive(&source, &destination)
        .map_err(|error| format!("Could not copy the extension: {error}"))?;

    if reveal {
        reveal_in_file_manager(&destination)?;
    }

    Ok(destination.to_string_lossy().into_owned())
}

fn copy_dir_recursive(from: &std::path::Path, to: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn reveal_in_file_manager(path: &std::path::Path) -> Result<(), String> {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "explorer"
    } else {
        "xdg-open"
    };
    std::process::Command::new(opener)
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not open the folder: {error}"))
}

/// Lists each browser profile and whether the Gramflow extension is loaded in it.
#[tauri::command]
async fn get_browser_extension_profiles()
-> Result<Vec<crate::browser_extension::BrowserProfileStatus>, String> {
    tokio::task::spawn_blocking(crate::browser_extension::detect_profiles)
        .await
        .map_err(|error| error.to_string())
}

/// Open the extensions page of a supported browser (`chrome`, `edge`, `brave`, `opera`, `firefox`), optionally in a
/// specific profile (the profile directory name, e.g. `Profile 1`).
#[tauri::command]
fn open_browser_extensions_page(
    browser: String,
    profile_directory: Option<String>,
    extension_id: Option<String>,
) -> Result<(), String> {
    let (app_name, url) = match browser.as_str() {
        "chrome" => ("Google Chrome", "chrome://extensions"),
        "edge" => ("Microsoft Edge", "edge://extensions"),
        "brave" => ("Brave Browser", "brave://extensions"),
        "opera" => ("Opera", "opera://extensions"),
        "firefox" => ("Firefox", "about:debugging#/runtime/this-firefox"),
        other => return Err(format!("Unsupported browser: {other}")),
    };

    if !cfg!(target_os = "macos") {
        return Err("Opening the extensions page is only supported on macOS for now.".into());
    }

    // With an extension id, open that extension's own page, which has Remove and Reload buttons.
    let details_url;
    let url = match extension_id
        .as_deref()
        .filter(|id| !id.is_empty() && browser != "firefox")
    {
        Some(id) => {
            details_url = format!("{url}/?id={id}");
            details_url.as_str()
        }
        None => url,
    };

    let mut command = std::process::Command::new("open");
    // Only Chrome-family browsers take a profile flag.
    match profile_directory.filter(|profile| !profile.is_empty() && browser != "firefox") {
        // `-n` is required so `--args` reach an already-running browser.
        Some(profile) => command
            .args(["-n", "-a", app_name, "--args"])
            .arg(format!("--profile-directory={profile}"))
            .arg(url),
        None => command.args(["-a", app_name, url]),
    };

    command
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not open {app_name}: {error}"))
}
