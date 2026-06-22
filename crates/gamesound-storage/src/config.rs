use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub app: App,
    pub tui: Tui,
    pub audio: Audio,
    pub volume: Volume,
    pub monitor: Monitor,
    pub ducking: Ducking,
    pub hotkeys: Hotkeys,
    #[serde(default)]
    pub desktop: Desktop,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    pub first_run: bool,
    pub active_profile: String,
    pub log_level: String,
    pub auto_save: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tui {
    pub theme: String,
    #[serde(default)]
    pub language: Language,
    pub tick_rate_ms: u64,
    pub show_level_meter: bool,
    pub confirm_on_delete: bool,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    #[default]
    English,
    Chinese,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Audio {
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: u32,
    pub playback_mode: String,
    pub devices: Devices,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Devices {
    pub mic: Option<String>,
    pub output: Option<String>,
    pub monitor: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    pub mic: f32,
    pub sfx: f32,
    pub monitor: f32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monitor {
    pub enabled: bool,
    pub mode: MonitorMode,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MonitorMode {
    SfxOnly,
    FullMix,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ducking {
    pub enabled: bool,
    pub ratio: f32,
    pub attack_ms: u32,
    pub release_ms: u32,
    pub release_delay_ms: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkeys {
    pub enabled: bool,
    pub stop_all: String,
    pub toggle_mic: String,
    pub toggle_sfx: String,
    #[serde(default = "default_toggle_monitor")]
    pub toggle_monitor: String,
    #[serde(default = "default_sfx_up")]
    pub sfx_volume_up: String,
    #[serde(default = "default_sfx_down")]
    pub sfx_volume_down: String,
    #[serde(default = "default_mic_up")]
    pub mic_volume_up: String,
    #[serde(default = "default_mic_down")]
    pub mic_volume_down: String,
    #[serde(default = "default_monitor_up")]
    pub monitor_volume_up: String,
    #[serde(default = "default_monitor_down")]
    pub monitor_volume_down: String,
    #[serde(default = "default_switch_profile")]
    pub switch_profile: String,
}
fn default_toggle_monitor() -> String {
    "ctrl+alt+l".into()
}
fn default_sfx_up() -> String {
    "ctrl+alt+up".into()
}
fn default_sfx_down() -> String {
    "ctrl+alt+down".into()
}
fn default_mic_up() -> String {
    "ctrl+alt+right".into()
}
fn default_mic_down() -> String {
    "ctrl+alt+left".into()
}
fn default_monitor_up() -> String {
    "ctrl+shift+up".into()
}
fn default_monitor_down() -> String {
    "ctrl+shift+down".into()
}
fn default_switch_profile() -> String {
    "ctrl+alt+p".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Desktop {
    #[serde(default = "default_close_behavior")]
    pub close_behavior: CloseBehavior,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CloseBehavior {
    #[default]
    Ask,
    MinimizeToTray,
    Quit,
}

fn default_close_behavior() -> CloseBehavior {
    CloseBehavior::Ask
}

impl Default for Desktop {
    fn default() -> Self {
        Self {
            close_behavior: CloseBehavior::Ask,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app: App {
                first_run: true,
                active_profile: "default".into(),
                log_level: "info".into(),
                auto_save: true,
            },
            tui: Tui {
                theme: "default".into(),
                language: Language::English,
                tick_rate_ms: 100,
                show_level_meter: true,
                confirm_on_delete: true,
            },
            audio: Audio {
                sample_rate: 48000,
                channels: 2,
                buffer_size: 512,
                playback_mode: "overlay".into(),
                devices: Devices::default(),
            },
            volume: Volume {
                mic: 0.9,
                sfx: 0.8,
                monitor: 0.6,
            },
            monitor: Monitor {
                enabled: false,
                mode: MonitorMode::SfxOnly,
            },
            ducking: Ducking {
                enabled: true,
                ratio: 0.4,
                attack_ms: 50,
                release_ms: 300,
                release_delay_ms: 200,
            },
            hotkeys: Hotkeys {
                enabled: true,
                stop_all: "ctrl+alt+s".into(),
                toggle_mic: "ctrl+alt+m".into(),
                toggle_sfx: "ctrl+alt+x".into(),
                toggle_monitor: default_toggle_monitor(),
                sfx_volume_up: default_sfx_up(),
                sfx_volume_down: default_sfx_down(),
                mic_volume_up: default_mic_up(),
                mic_volume_down: default_mic_down(),
                monitor_volume_up: default_monitor_up(),
                monitor_volume_down: default_monitor_down(),
                switch_profile: default_switch_profile(),
            },
            desktop: Desktop::default(),
        }
    }
}
pub struct ConfigStore {
    root: PathBuf,
}
impl ConfigStore {
    pub fn for_current_user() -> Result<Self> {
        let dirs = ProjectDirs::from("com", "GameSound", "GameSound")
            .context("cannot determine user config directory")?;
        Ok(Self {
            root: dirs.data_local_dir().to_path_buf(),
        })
    }
    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn config_path(&self) -> PathBuf {
        self.root.join("config.toml")
    }
    pub fn db_path(&self) -> PathBuf {
        self.root.join("gamesound.db")
    }
    pub fn sounds_path(&self) -> PathBuf {
        self.root.join("sounds")
    }
    pub fn logs_path(&self) -> PathBuf {
        self.root.join("logs")
    }
    pub fn initialise(&self) -> Result<()> {
        for dir in ["", "sounds", "backups", "logs", "profiles"] {
            fs::create_dir_all(self.root.join(dir))?;
        }
        Ok(())
    }
    pub fn load(&self) -> Result<AppConfig> {
        self.initialise()?;
        let path = self.config_path();
        if !path.exists() {
            let c = AppConfig::default();
            self.save(&c)?;
            return Ok(c);
        }
        let text = fs::read_to_string(&path)?;
        match toml::from_str(&text) {
            Ok(c) => Ok(c),
            Err(e) => {
                let backup = path.with_extension(format!(
                    "corrupt-{}.toml",
                    chrono::Local::now().format("%Y%m%d%H%M%S")
                ));
                fs::rename(&path, &backup).context("could not preserve corrupt config")?;
                let c = AppConfig::default();
                self.save(&c)?;
                Err(anyhow::anyhow!(
                    "configuration was invalid and reset (backup: {}): {e}",
                    backup.display()
                ))
            }
        }
    }
    pub fn save(&self, c: &AppConfig) -> Result<()> {
        self.initialise()?;
        let destination = self.config_path();
        let temporary = destination.with_extension("toml.tmp");
        fs::write(&temporary, toml::to_string_pretty(c)?)?;
        fs::rename(&temporary, &destination)
            .with_context(|| format!("cannot atomically save {}", destination.display()))?;
        Ok(())
    }
    /// Saves a self-contained configuration/database snapshot before destructive changes.
    pub fn backup(&self) -> Result<PathBuf> {
        self.initialise()?;
        let destination = self
            .root
            .join("backups")
            .join(chrono::Local::now().format("%Y%m%d-%H%M%S").to_string());
        fs::create_dir_all(&destination)?;
        for name in ["config.toml", "gamesound.db"] {
            let source = self.root.join(name);
            if source.exists() {
                fs::copy(&source, destination.join(name))?;
            }
        }
        Ok(destination)
    }
    pub fn restore_backup(&self, backup: &Path) -> Result<()> {
        for name in ["config.toml", "gamesound.db"] {
            let source = backup.join(name);
            if source.exists() {
                fs::copy(&source, self.root.join(name))
                    .with_context(|| format!("cannot restore {name}"))?;
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn writes_and_reads() {
        let d = std::env::temp_dir().join(format!("gamesound-test-{}", std::process::id()));
        let s = ConfigStore::at(&d);
        let c = s.load().unwrap();
        s.save(&c).unwrap();
        assert_eq!(s.load().unwrap().audio.sample_rate, 48000);
        let _ = std::fs::remove_dir_all(d);
    }
    #[test]
    fn backup_copies_config_and_database() {
        let d = std::env::temp_dir().join(format!("gamesound-backup-test-{}", std::process::id()));
        let store = ConfigStore::at(&d);
        store.save(&AppConfig::default()).unwrap();
        std::fs::write(store.db_path(), b"database").unwrap();
        let backup = store.backup().unwrap();
        assert!(backup.join("config.toml").is_file());
        assert_eq!(
            std::fs::read(backup.join("gamesound.db")).unwrap(),
            b"database"
        );
        let _ = std::fs::remove_dir_all(d);
    }
    #[test]
    fn language_selection_persists() {
        let d =
            std::env::temp_dir().join(format!("gamesound-language-test-{}", std::process::id()));
        let store = ConfigStore::at(&d);
        let mut config = AppConfig::default();
        config.tui.language = Language::Chinese;
        store.save(&config).unwrap();
        assert_eq!(store.load().unwrap().tui.language, Language::Chinese);
        let _ = std::fs::remove_dir_all(d);
    }
}
