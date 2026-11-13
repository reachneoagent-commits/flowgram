//! Functions to manage the main windows involved in Harper Desktop

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
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

/// Open the editor window, focusing it if it already exists.
pub fn show_editor_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("editor") {
        return show_window(window);
    }

    let window = WebviewWindowBuilder::new(app, "editor", WebviewUrl::App("index.html".into()))
        .title("Flowgram")
        .inner_size(800.0, 600.0)
        .visible(!START_HIDDEN)
        .build()?;

    show_window(window)
}

/// Open the Humanize window, focusing it if it already exists.
pub fn show_humanize_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("humanize") {
        return show_window(window);
    }

    let window = WebviewWindowBuilder::new(app, "humanize", WebviewUrl::App("index.html".into()))
        .title("Flowgram Humanize")
        .inner_size(980.0, 640.0)
        .min_inner_size(640.0, 420.0)
        .center()
        .visible(!START_HIDDEN)
        .build()?;

    show_window(window)
}

/// Picks a settings window size that suits the screen: most of the display on small screens,
/// capped on large ones, and never smaller than the layout needs.
fn settings_window_size(app: &tauri::AppHandle) -> (f64, f64) {
    const MIN: (f64, f64) = (920.0, 680.0);
    const MAX: (f64, f64) = (1280.0, 960.0);

    let Some(monitor) = app.primary_monitor().ok().flatten() else {
        return MIN;
    };
    let scale = monitor.scale_factor();
    let screen_width = monitor.size().width as f64 / scale;
    let screen_height = monitor.size().height as f64 / scale;

    // Leave room for the menu bar and Dock, then take most of what is left.
    let width = (screen_width * 0.85)
        .clamp(MIN.0, MAX.0)
        .min(screen_width - 40.0);
    let height = (screen_height * 0.88)
        .clamp(MIN.1, MAX.1)
        .min(screen_height - 100.0);

    (width, height)
}

/// Open the settings window, focusing it if it already exists.
pub fn show_settings_window(app: &tauri::AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("settings") {
        return show_window(window);
    }

    let (width, height) = settings_window_size(app);
    let window = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
        .title("Flowgram Settings")
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
