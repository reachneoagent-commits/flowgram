//! Humanizer preferences saved in Flowgram's main config file.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HumanizerSettings {
    /// The user chose "Skip for now" during setup. Settings still offers setup.
    pub skipped: bool,
    /// Id of the model size used for rewrites.
    pub active: Option<String>,
    /// Minutes of inactivity before the engine unloads and frees memory.
    pub idle_minutes: u64,
    /// Flag rewrites whose numbers differ from the draft.
    pub warn_numbers: bool,
}

impl Default for HumanizerSettings {
    fn default() -> Self {
        Self {
            skipped: false,
            active: None,
            idle_minutes: 10,
            warn_numbers: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_safe() {
        let settings = HumanizerSettings::default();
        assert!(!settings.skipped);
        assert_eq!(settings.active, None);
        assert_eq!(settings.idle_minutes, 10);
        assert!(settings.warn_numbers);
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let settings: HumanizerSettings = serde_json::from_str(r#"{"skipped":true}"#).unwrap();
        assert!(settings.skipped);
        assert_eq!(settings.idle_minutes, 10);
        assert!(settings.warn_numbers);
    }

    #[test]
    fn round_trips() {
        let settings = HumanizerSettings {
            skipped: true,
            active: Some("q6".into()),
            idle_minutes: 0,
            warn_numbers: false,
        };
        let json = serde_json::to_string(&settings).unwrap();
        assert_eq!(
            serde_json::from_str::<HumanizerSettings>(&json).unwrap(),
            settings
        );
    }
}
