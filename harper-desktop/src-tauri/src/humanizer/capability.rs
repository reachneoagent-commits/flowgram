//! Decides whether this computer can run Humanizer, and which model sizes make sense.
//! `assess` is pure so every row of the policy table is unit tested.

use super::models::{TIERS, Tier};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Os {
    Mac,
    Windows,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Arch {
    Arm64,
    X64,
    Other,
}

/// What we measured about this computer.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Facts {
    pub os: Os,
    pub arch: Arch,
    pub total_ram_gb: f64,
    pub free_disk_gb: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Level {
    /// Runs well; offer setup.
    Ready,
    /// Runs, with caveats the user should read first.
    Limited,
    /// Cannot run; show the reason and no download.
    Unavailable,
}

#[derive(Debug, Clone, Serialize)]
pub struct TierOption {
    pub id: &'static str,
    pub label: &'static str,
    pub blurb: &'static str,
    pub size_bytes: u64,
    pub min_ram_gb: u64,
    /// Enough memory to run it.
    pub fits_memory: bool,
    /// Enough free disk to download it, with some headroom.
    pub fits_disk: bool,
    pub recommended: bool,
    pub installed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Verdict {
    pub level: Level,
    /// One plain sentence for the title area.
    pub headline: String,
    /// Why, or what to expect. Empty when there is nothing to add.
    pub detail: String,
    pub facts: Facts,
    pub options: Vec<TierOption>,
}

const MIN_RAM_GB: f64 = 8.0;
/// Free disk required beyond the download itself.
const DISK_HEADROOM_GB: f64 = 2.0;

pub fn detect() -> Facts {
    use sysinfo::{Disks, System};

    // If the system can't be read, report zero so the verdict is "unavailable" rather than a guess.
    let total_ram_gb = System::new()
        .map(|mut system| {
            system.refresh_memory();
            system.total_memory() as f64 / 1e9
        })
        .unwrap_or(0.0);

    let disks = Disks::new_with_refreshed_list().ok();
    let target = Tier::models_dir().or_else(dirs::config_dir);
    let free = target
        .zip(disks)
        .and_then(|(dir, disks)| {
            disks
                .iter()
                .filter(|disk| dir.starts_with(disk.mount_point()))
                .max_by_key(|disk| disk.mount_point().as_os_str().len())
                .map(|disk| disk.available_space())
        })
        .unwrap_or(0);

    Facts {
        os: if cfg!(target_os = "macos") {
            Os::Mac
        } else if cfg!(target_os = "windows") {
            Os::Windows
        } else {
            Os::Other
        },
        arch: if cfg!(target_arch = "aarch64") {
            Arch::Arm64
        } else if cfg!(target_arch = "x86_64") {
            Arch::X64
        } else {
            Arch::Other
        },
        total_ram_gb,
        free_disk_gb: free as f64 / 1e9,
    }
}

pub fn assess(facts: Facts, installed: impl Fn(Tier) -> bool) -> Verdict {
    let options = |facts: &Facts| -> Vec<TierOption> {
        let mut recommended_taken = false;
        TIERS
            .iter()
            .map(|info| {
                let fits_memory = facts.total_ram_gb >= info.min_ram_gb as f64;
                let fits_disk =
                    facts.free_disk_gb >= info.size_bytes as f64 / 1e9 + DISK_HEADROOM_GB;
                let recommended = fits_memory && !recommended_taken;
                recommended_taken |= recommended;
                TierOption {
                    id: info.id,
                    label: info.label,
                    blurb: info.blurb,
                    size_bytes: info.size_bytes,
                    min_ram_gb: info.min_ram_gb,
                    fits_memory,
                    fits_disk,
                    recommended,
                    installed: installed(info.tier),
                }
            })
            .collect()
    };

    let unavailable = |facts: Facts, detail: &str| Verdict {
        level: Level::Unavailable,
        headline: "Humanizer isn't available on this computer".into(),
        detail: detail.into(),
        options: Vec::new(),
        facts,
    };

    match (facts.os, facts.arch) {
        (Os::Mac, Arch::Arm64) | (Os::Windows, Arch::X64) => {}
        (Os::Mac, _) => {
            return unavailable(
                facts,
                "Humanizer needs a Mac with Apple silicon. Intel Macs are too slow to run it.",
            );
        }
        (Os::Windows, _) => {
            return unavailable(
                facts,
                "Humanizer needs a 64-bit Intel or AMD Windows PC. Windows on ARM isn't supported yet.",
            );
        }
        _ => {
            return unavailable(facts, "Humanizer works on macOS and Windows.");
        }
    }

    if facts.total_ram_gb < MIN_RAM_GB {
        let detail = format!(
            "It needs at least 8 GB of memory, and this computer has {:.0} GB.",
            facts.total_ram_gb
        );
        return unavailable(facts, &detail);
    }

    let options = options(&facts);
    if !options
        .iter()
        .any(|option| option.fits_memory && option.fits_disk)
    {
        let smallest = TIERS[TIERS.len() - 1].size_bytes as f64 / 1e9 + DISK_HEADROOM_GB;
        let detail = format!(
            "The smallest model needs about {:.0} GB of free disk space, and {:.0} GB is free.",
            smallest, facts.free_disk_gb
        );
        return unavailable(facts, &detail);
    }

    let recommended_small = options
        .iter()
        .find(|option| option.recommended)
        .is_some_and(|option| option.min_ram_gb <= 10);

    let (level, headline, detail) = if recommended_small {
        (
            Level::Limited,
            "Humanizer can run here, with limits",
            "This computer can run the smaller models. Rewrites may be slower and miss a few facts, so check numbers and names.".to_string(),
        )
    } else if facts.os == Os::Windows {
        (
            Level::Ready,
            "Your PC can run Humanizer",
            "Speed depends on your graphics card. Without a supported one, rewrites run on the processor and take longer.".to_string(),
        )
    } else {
        (Level::Ready, "Your Mac can run Humanizer", String::new())
    };

    Verdict {
        level,
        headline: headline.into(),
        detail,
        facts,
        options,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(os: Os, arch: Arch, ram: f64, disk: f64) -> Facts {
        Facts {
            os,
            arch,
            total_ram_gb: ram,
            free_disk_gb: disk,
        }
    }

    fn verdict(os: Os, arch: Arch, ram: f64, disk: f64) -> Verdict {
        assess(facts(os, arch, ram, disk), |_| false)
    }

    fn recommended(verdict: &Verdict) -> Option<&'static str> {
        verdict.options.iter().find(|o| o.recommended).map(|o| o.id)
    }

    #[test]
    fn big_apple_silicon_gets_the_best_model() {
        let v = verdict(Os::Mac, Arch::Arm64, 128.0, 400.0);
        assert_eq!(v.level, Level::Ready);
        assert_eq!(recommended(&v), Some("q8"));
        assert!(v.options.iter().all(|o| o.fits_memory));
    }

    #[test]
    fn apple_silicon_tiers_follow_memory() {
        assert_eq!(
            recommended(&verdict(Os::Mac, Arch::Arm64, 32.0, 100.0)),
            Some("q8")
        );
        assert_eq!(
            recommended(&verdict(Os::Mac, Arch::Arm64, 18.0, 100.0)),
            Some("q6")
        );
        assert_eq!(
            recommended(&verdict(Os::Mac, Arch::Arm64, 16.0, 100.0)),
            Some("q6")
        );
        assert_eq!(
            recommended(&verdict(Os::Mac, Arch::Arm64, 12.0, 100.0)),
            Some("q4")
        );
    }

    #[test]
    fn eight_to_eleven_gb_is_limited() {
        let v = verdict(Os::Mac, Arch::Arm64, 8.0, 100.0);
        assert_eq!(v.level, Level::Limited);
        assert_eq!(recommended(&v), Some("q2"));
        let v = verdict(Os::Mac, Arch::Arm64, 10.0, 100.0);
        assert_eq!(v.level, Level::Limited);
        assert_eq!(recommended(&v), Some("q3"));
    }

    #[test]
    fn under_eight_gb_is_unavailable_with_a_reason() {
        let v = verdict(Os::Mac, Arch::Arm64, 7.5, 100.0);
        assert_eq!(v.level, Level::Unavailable);
        assert!(v.detail.contains("8 GB"));
        assert!(v.options.is_empty());
    }

    #[test]
    fn intel_mac_is_unavailable() {
        let v = verdict(Os::Mac, Arch::X64, 64.0, 400.0);
        assert_eq!(v.level, Level::Unavailable);
        assert!(v.detail.contains("Apple silicon"));
    }

    #[test]
    fn windows_on_arm_is_unavailable() {
        let v = verdict(Os::Windows, Arch::Arm64, 32.0, 400.0);
        assert_eq!(v.level, Level::Unavailable);
        assert!(v.detail.contains("ARM"));
    }

    #[test]
    fn windows_x64_is_ready_with_a_speed_note() {
        let v = verdict(Os::Windows, Arch::X64, 32.0, 400.0);
        assert_eq!(v.level, Level::Ready);
        assert!(v.detail.contains("graphics card"));
    }

    #[test]
    fn other_systems_are_unavailable() {
        let v = verdict(Os::Other, Arch::X64, 64.0, 400.0);
        assert_eq!(v.level, Level::Unavailable);
    }

    #[test]
    fn too_little_disk_for_any_model_is_unavailable() {
        let v = verdict(Os::Mac, Arch::Arm64, 64.0, 3.0);
        assert_eq!(v.level, Level::Unavailable);
        assert!(v.detail.contains("free disk"));
    }

    #[test]
    fn disk_only_blocks_sizes_that_do_not_fit() {
        let v = verdict(Os::Mac, Arch::Arm64, 64.0, 9.0);
        assert_ne!(v.level, Level::Unavailable);
        let fits: Vec<_> = v.options.iter().map(|o| (o.id, o.fits_disk)).collect();
        assert_eq!(
            fits,
            vec![
                ("q8", false),
                ("q6", false),
                ("q4", false),
                ("q3", true),
                ("q2", true)
            ]
        );
    }

    #[test]
    fn installed_state_is_reported_per_tier() {
        let v = assess(facts(Os::Mac, Arch::Arm64, 64.0, 400.0), |t| t == Tier::Q6);
        let installed: Vec<_> = v
            .options
            .iter()
            .filter(|o| o.installed)
            .map(|o| o.id)
            .collect();
        assert_eq!(installed, vec!["q6"]);
    }

    #[test]
    fn exactly_one_option_is_recommended() {
        let v = verdict(Os::Mac, Arch::Arm64, 64.0, 400.0);
        assert_eq!(v.options.iter().filter(|o| o.recommended).count(), 1);
    }

    #[test]
    fn detect_reports_this_machine() {
        let facts = detect();
        assert!(facts.total_ram_gb > 0.0);
        assert!(facts.free_disk_gb > 0.0);
    }
}
