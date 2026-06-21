use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopSettings {
    pub mic_device: Option<String>,
    pub output_device: Option<String>,
    pub monitor_device: Option<String>,
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: u32,
    pub mic_volume: f32,
    pub sfx_volume: f32,
    pub monitor_volume: f32,
    pub monitor_enabled: bool,
    pub monitor_mode: String,
    pub ducking_enabled: bool,
    pub duck_ratio: f32,
    pub duck_attack_ms: u32,
    pub duck_release_ms: u32,
    pub duck_release_delay_ms: u32,
    pub hotkeys_enabled: bool,
    pub hotkey_stop_all: String,
    pub hotkey_toggle_mic: String,
    pub hotkey_toggle_sfx: String,
    pub hotkey_toggle_monitor: String,
    pub theme: String,
    pub language: String,
    pub log_level: String,
    pub config_dir: String,
    pub log_dir: String,
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<DesktopSettings, String> {
    let config = state.store.load().map_err(|e| e.to_string())?;
    let settings = state.mixer_settings.lock().unwrap();

    Ok(DesktopSettings {
        mic_device: config.audio.devices.mic.clone(),
        output_device: config.audio.devices.output.clone(),
        monitor_device: config.audio.devices.monitor.clone(),
        sample_rate: config.audio.sample_rate,
        channels: config.audio.channels,
        buffer_size: config.audio.buffer_size,
        mic_volume: settings.mic_volume,
        sfx_volume: settings.sfx_volume,
        monitor_volume: settings.monitor_volume,
        monitor_enabled: config.monitor.enabled,
        monitor_mode: match config.monitor.mode {
            gamesound_storage::config::MonitorMode::SfxOnly => "sfx_only".into(),
            gamesound_storage::config::MonitorMode::FullMix => "full_mix".into(),
            gamesound_storage::config::MonitorMode::Off => "off".into(),
        },
        ducking_enabled: settings.ducking,
        duck_ratio: settings.duck_ratio,
        duck_attack_ms: settings.duck_attack_ms,
        duck_release_ms: settings.duck_release_ms,
        duck_release_delay_ms: settings.duck_release_delay_ms,
        hotkeys_enabled: config.hotkeys.enabled,
        hotkey_stop_all: config.hotkeys.stop_all.clone(),
        hotkey_toggle_mic: config.hotkeys.toggle_mic.clone(),
        hotkey_toggle_sfx: config.hotkeys.toggle_sfx.clone(),
        hotkey_toggle_monitor: config.hotkeys.toggle_monitor.clone(),
        theme: config.tui.theme.clone(),
        language: match config.tui.language {
            gamesound_storage::config::Language::English => "english".into(),
            gamesound_storage::config::Language::Chinese => "chinese".into(),
        },
        log_level: config.app.log_level.clone(),
        config_dir: state.store.root().to_string_lossy().into(),
        log_dir: state.store.logs_path().to_string_lossy().into(),
    })
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsParams {
    pub theme: Option<String>,
    pub language: Option<String>,
    pub log_level: Option<String>,
    pub monitor_mode: Option<String>,
    pub hotkeys_enabled: Option<bool>,
}

#[tauri::command]
pub fn update_settings(state: State<AppState>, params: UpdateSettingsParams) -> Result<(), String> {
    let mut config = state.store.load().map_err(|e| e.to_string())?;

    if let Some(theme) = params.theme {
        config.tui.theme = theme;
    }
    if let Some(lang) = params.language {
        config.tui.language = match lang.as_str() {
            "chinese" => gamesound_storage::config::Language::Chinese,
            _ => gamesound_storage::config::Language::English,
        };
    }
    if let Some(level) = params.log_level {
        config.app.log_level = level;
    }
    if let Some(mode) = params.monitor_mode {
        config.monitor.mode = match mode.as_str() {
            "full_mix" => gamesound_storage::config::MonitorMode::FullMix,
            "off" => gamesound_storage::config::MonitorMode::Off,
            _ => gamesound_storage::config::MonitorMode::SfxOnly,
        };
    }
    if let Some(enabled) = params.hotkeys_enabled {
        config.hotkeys.enabled = enabled;
    }

    state.store.save(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_config(state: State<AppState>) -> Result<String, String> {
    let config = state.store.load().map_err(|e| e.to_string())?;
    toml::to_string_pretty(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_config(state: State<AppState>, toml_content: String) -> Result<(), String> {
    let config: gamesound_storage::config::AppConfig =
        toml::from_str(&toml_content).map_err(|e| format!("Invalid config format: {}", e))?;
    state.store.save(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reset_config(state: State<AppState>) -> Result<(), String> {
    let default_config = gamesound_storage::config::AppConfig::default();
    state.store.save(&default_config).map_err(|e| e.to_string())
}

fn open_in_file_manager(path: &std::path::Path) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn open_config_dir(state: State<AppState>) -> Result<(), String> {
    let path = state.store.root().to_path_buf();
    open_in_file_manager(&path)
}

#[tauri::command]
pub fn open_log_dir(state: State<AppState>) -> Result<(), String> {
    let path = state.store.logs_path();
    if !path.exists() {
        std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    }
    open_in_file_manager(&path)
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfileInfo {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub is_active: bool,
}

#[tauri::command]
pub fn get_profile_info(state: State<AppState>) -> Result<ProfileInfo, String> {
    let library = state.library.lock().unwrap();
    let profile = library.active_profile().map_err(|e| e.to_string())?;
    Ok(ProfileInfo {
        id: profile.id,
        name: profile.name,
        description: profile.description,
        is_active: profile.is_active,
    })
}
