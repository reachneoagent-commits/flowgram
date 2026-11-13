//! Functions to manage the main windows involved in Harper Desktop

use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;
use tracing::error;

/// Synchronize macOS Dock visibility with the application's open windows.
///
/// Minimized windows still count as open. Exclude a destroyed window explicitly because
/// it may still be registered when its destruction event is delivered. Highlighter
/// windows belong to a separate process and do not affect this policy.
#[cfg(target_os = "macos")]
pub fn sync_dock_visibility(app: &AppHandle, destroyed_label: Option<&str>) -> tauri::Result<()> {
    let windows = app.webview_windows();
    let policy = activation_policy_for_windows(windows.keys().map(String::as_str), destroyed_label);
    let restore_icon = matches!(policy, tauri::ActivationPolicy::Regular);
    app.set_activation_policy(policy)?;

    if restore_icon {
        let _ = app
            .run_on_main_thread(restore_dock_icon)
            .inspect_err(|err| error!("Could not schedule Dock icon restoration: {err}"));
    }

    Ok(())
}

/// Reapply the bundled icon after restoring the app's Dock presence.
///
/// Tauri assigns the development-mode icon only at startup; recreating the Dock
/// entry can lose that image. Run this on the main thread after selecting Regular
/// policy, loading the embedded icon rather than relying on cached native state.
#[cfg(target_os = "macos")]
fn restore_dock_icon() {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::NSData;

    let Some(mtm) = MainThreadMarker::new() else {
        error!("Dock icon restoration must run on the main thread");
        return;
    };
    let data = NSData::with_bytes(include_bytes!("../icons/icon.icns"));
    let Some(image) = NSImage::initWithData(mtm.alloc(), &data) else {
        error!("Could not decode the bundled Dock icon");
        return;
    };

    // SAFETY: The setter receives a non-nil image, and AppKit is accessed on the main thread.
    unsafe { NSApplication::sharedApplication(mtm).setApplicationIconImage(Some(&image)) };
}

/// Select the activation policy from open window labels, ignoring a destroyed window.
#[cfg(target_os = "macos")]
fn activation_policy_for_windows<'a>(
    mut labels: impl Iterator<Item = &'a str>,
    destroyed_label: Option<&str>,
) -> tauri::ActivationPolicy {
    if labels.any(|label| Some(label) != destroyed_label) {
        tauri::ActivationPolicy::Regular
    } else {
        tauri::ActivationPolicy::Accessory
    }
}

/// Present a registered window after restoring its macOS Dock presence.
///
/// New windows are built hidden so activation policy changes precede showing and
/// focusing them. Failed window creation therefore leaves the policy unchanged.
fn show_window(window: WebviewWindow) -> tauri::Result<()> {
    #[cfg(target_os = "macos")]
    sync_dock_visibility(window.app_handle(), None)?;

    window.show()?;
    window.set_focus()
}

/// Build windows hidden only on macOS, where the Dock policy must change before they appear.
/// On Windows a WebView2 window created hidden can stay blank white after `show()`, so it is
/// created visible there instead.
const START_HIDDEN: bool = cfg!(target_os = "macos");

/// Which tab the editor window opens on.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EditorTab {
    Check,
    Humanize,
}

impl EditorTab {
    fn name(self) -> &'static str {
        match self {
            EditorTab::Check => "check",
            EditorTab::Humanize => "humanize",
        }
    }
}

/// Open the editor window on `tab`, focusing it if it already exists.
///
/// The editor holds both the grammar checker and Humanizer as tabs. A new window learns its tab
/// from its URL; an open one is told through an event.
pub fn show_editor_tab(app: &tauri::AppHandle, tab: EditorTab) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("editor") {
        let _ = app.emit("editor-tab", tab.name());
        return show_window(window);
    }

    // The tab goes in as a script rather than a `?tab=` query: app URLs are file paths, so a query
    // string would make the page fail to load (a 404).
    let window = WebviewWindowBuilder::new(app, "editor", WebviewUrl::App("index.html".into()))
        .initialization_script(format!("window.__GRAMFLOW_TAB__ = '{}';", tab.name()))
        .title("Gramflow")
        .theme(saved_theme(app))
        .inner_size(1100.0, 760.0)
        .min_inner_size(720.0, 480.0)
        .center()
        .visible(!START_HIDDEN)
        .build()?;

    show_window(window)
}

/// Open the editor window on the grammar checker.
pub fn show_editor_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    show_editor_tab(app, EditorTab::Check)
}

/// Open the editor window on Humanizer.
pub fn show_humanize_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    show_editor_tab(app, EditorTab::Humanize)
}

/// The window frame follows the app's saved theme so a dark app doesn't wear a light title bar.
/// "system" (or an unreadable config) leaves the choice to the operating system.
pub fn saved_theme(app: &tauri::AppHandle) -> Option<tauri::Theme> {
    let config = app.try_state::<std::sync::Arc<tokio::sync::Mutex<crate::config::Config>>>()?;
    let config = config.try_lock().ok()?;
    match config.theme.as_str() {
        "dark" => Some(tauri::Theme::Dark),
        "light" => Some(tauri::Theme::Light),
        _ => None,
    }
}

fn onboarding_pending(app: &tauri::AppHandle) -> bool {
    app.try_state::<std::sync::Arc<tokio::sync::Mutex<crate::config::Config>>>()
        .and_then(|config| {
            config
                .try_lock()
                .ok()
                .map(|config| !config.onboarding_completed)
        })
        .unwrap_or(false)
}

/// Picks a settings window size that suits the screen: most of the display on small screens,
/// capped on large ones, and never smaller than the layout needs.
pub fn settings_window_size(app: &tauri::AppHandle) -> (f64, f64) {
    fit_to_screen(app, (0.85, 0.88), (920.0, 680.0), (1280.0, 960.0))
}

/// Setup gets a larger window than Settings: tall enough that nearly every step fits without
/// scrolling, and still never bigger than the screen.
pub fn onboarding_window_size(app: &tauri::AppHandle) -> (f64, f64) {
    fit_to_screen(app, (0.92, 0.94), (1000.0, 740.0), (1500.0, 1100.0))
}

/// Takes `fractions` of the screen, kept between `min` and `max`, and always leaves room for the
/// menu bar and Dock. Sizes are in logical points.
fn fit_to_screen(
    app: &tauri::AppHandle,
    fractions: (f64, f64),
    min: (f64, f64),
    max: (f64, f64),
) -> (f64, f64) {
    let Some(monitor) = app.primary_monitor().ok().flatten() else {
        return min;
    };
    let scale = monitor.scale_factor();
    let screen_width = monitor.size().width as f64 / scale;
    let screen_height = monitor.size().height as f64 / scale;

    let width = (screen_width * fractions.0)
        .clamp(min.0, max.0)
        .min(screen_width - 40.0);
    let height = (screen_height * fractions.1)
        .clamp(min.1, max.1)
        .min(screen_height - 100.0);

    (width, height)
}

/// Open the settings window, focusing it if it already exists.
pub fn show_settings_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("settings") {
        return show_window(window);
    }

    // Setup gets its own screen-fitted size; Settings settles on its usual one when setup is done.
    let (width, height) = if onboarding_pending(app) {
        onboarding_window_size(app)
    } else {
        settings_window_size(app)
    };
    let window = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
        .theme(saved_theme(app))
        .title("Gramflow Settings")
        .inner_size(width, height)
        .min_inner_size(780.0, 520.0)
        .center()
        .visible(!START_HIDDEN)
        .build()?;

    show_window(window)
}

/// Open the browser to an issue report page.
pub fn open_issue_report(app: &AppHandle) {
    let _ = app
        .opener()
        .open_url(
            "https://github.com/reachneoagent-commits/gramflow/issues/new/choose",
            None::<&str>,
        )
        .inspect_err(|err| error!("failed to open issue report URL: {err}"));
}
