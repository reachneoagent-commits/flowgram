//! Reads the user's selected text in any app, and puts a rewrite back, by using the clipboard
//! the way a person would: copy before, paste after. Needs the same Accessibility permission
//! Flowgram already asks for on macOS.

use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::time::Duration;

#[cfg(target_os = "macos")]
const COMMAND_KEY: Key = Key::Meta;
#[cfg(not(target_os = "macos"))]
const COMMAND_KEY: Key = Key::Control;

/// Shortcut that starts a rewrite of the current selection.
pub const SHORTCUT: &str = if cfg!(target_os = "macos") {
    "Control+Super+H"
} else {
    "Control+Alt+H"
};

fn keyboard() -> Result<Enigo, String> {
    Enigo::new(&Settings::default()).map_err(|error| {
        format!(
            "Couldn't control the keyboard ({error}). Check Flowgram's Accessibility permission."
        )
    })
}

/// The shortcut's own modifier keys may still be down; let go of them so the app sees a plain
/// copy or paste.
fn release_modifiers(enigo: &mut Enigo) {
    for key in [Key::Control, Key::Meta, Key::Alt, Key::Shift] {
        let _ = enigo.key(key, Direction::Release);
    }
}

fn press_command_letter(enigo: &mut Enigo, letter: char) -> Result<(), String> {
    release_modifiers(enigo);
    std::thread::sleep(Duration::from_millis(60));
    let error = |error: enigo::InputError| error.to_string();
    enigo.key(COMMAND_KEY, Direction::Press).map_err(error)?;
    let clicked = enigo.key(Key::Unicode(letter), Direction::Click);
    let released = enigo.key(COMMAND_KEY, Direction::Release);
    clicked.map_err(error)?;
    released.map_err(error)
}

/// Copies whatever is selected in the frontmost app and returns it, leaving the clipboard as it
/// was. Returns `None` when nothing is selected.
pub fn capture_selection() -> Result<Option<String>, String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|error| error.to_string())?;
    let previous = clipboard.get_text().ok();

    // A marker tells "nothing was copied" apart from "the selection equals the old clipboard".
    let marker = format!("flowgram-{}", fastrand::u64(..));
    clipboard
        .set_text(marker.clone())
        .map_err(|error| error.to_string())?;

    let mut enigo = keyboard()?;
    press_command_letter(&mut enigo, 'c')?;

    let mut copied = None;
    for _ in 0..10 {
        std::thread::sleep(Duration::from_millis(40));
        if let Ok(text) = clipboard.get_text()
            && text != marker
        {
            copied = Some(text);
            break;
        }
    }

    match previous {
        Some(text) => clipboard.set_text(text).ok(),
        None => clipboard.clear().ok(),
    };
    Ok(copied.filter(|text| !text.trim().is_empty()))
}

/// Pastes `text` into whichever app is frontmost. The clipboard keeps `text` afterwards, so the
/// user can paste it again.
pub fn paste_text(text: &str) -> Result<(), String> {
    let mut clipboard = arboard::Clipboard::new().map_err(|error| error.to_string())?;
    clipboard
        .set_text(text.to_string())
        .map_err(|error| error.to_string())?;
    let mut enigo = keyboard()?;
    press_command_letter(&mut enigo, 'v')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shortcut_is_a_valid_accelerator() {
        assert!(SHORTCUT.contains("+H"));
        assert!(SHORTCUT.starts_with("Control+"));
    }
}
