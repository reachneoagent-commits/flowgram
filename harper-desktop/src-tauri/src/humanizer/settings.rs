//! Humanizer preferences saved in Gramflow's main config file.

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
    /// Look for approved model updates about once a day.
    pub auto_check_updates: bool,
    /// Unix seconds of the last update check, successful or not.
    pub last_update_check: Option<u64>,
}

impl Default for HumanizerSettings {
    fn default() -> Self {
        Self {
            skipped: false,
            active: None,
            idle_minutes: 10,
            warn_numbers: true,
            auto_check_updates: true,
            last_update_check: None,
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
        assert!(settings.auto_check_updates);
        assert_eq!(settings.last_update_check, None);
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
            auto_check_updates: false,
            last_update_check: Some(1_700_000_000),
        };
        let json = serde_json::to_string(&settings).unwrap();
        assert_eq!(
            serde_json::from_str::<HumanizerSettings>(&json).unwrap(),
            settings
        );
    }
}
