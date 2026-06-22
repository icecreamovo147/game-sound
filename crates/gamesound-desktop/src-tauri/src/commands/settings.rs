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
    pub close_behavior: String,
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<DesktopSettings, String> {
    tracing::info!(target: "gamesound_desktop::command", "called: get_settings");

    let config = state.store.load().map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "get_settings", error = %e, "failed to load config");
        e.to_string()
    })?;
    let settings = state.mixer_settings.lock().unwrap();

    tracing::debug!(target: "gamesound_desktop::settings", "settings read");
    tracing::debug!(target: "gamesound_desktop::command", "success: get_settings");

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
        close_behavior: match config.desktop.close_behavior {
            gamesound_storage::config::CloseBehavior::Ask => "ask".into(),
            gamesound_storage::config::CloseBehavior::MinimizeToTray => "minimize_to_tray".into(),
            gamesound_storage::config::CloseBehavior::Quit => "quit".into(),
        },
    })
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsParams {
    pub theme: Option<String>,
    pub language: Option<String>,
    pub log_level: Option<String>,
    pub monitor_mode: Option<String>,
    pub hotkeys_enabled: Option<bool>,
    pub close_behavior: Option<String>,
}

#[tauri::command]
pub fn update_settings(state: State<AppState>, params: UpdateSettingsParams) -> Result<(), String> {
    tracing::info!(target: "gamesound_desktop::command", "called: update_settings");

    let mut config = state.store.load().map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "update_settings", error = %e, "failed to load config");
        e.to_string()
    })?;

    let mut changes = Vec::new();

    if let Some(theme) = params.theme {
        config.tui.theme = theme.clone();
        changes.push(format!("theme={}", theme));
    }
    if let Some(lang) = params.language {
        config.tui.language = match lang.as_str() {
            "chinese" => gamesound_storage::config::Language::Chinese,
            _ => gamesound_storage::config::Language::English,
        };
        changes.push(format!("language={}", lang));
    }
    if let Some(level) = params.log_level {
        config.app.log_level = level.clone();
        changes.push(format!("log_level={}", level));
    }
    if let Some(mode) = params.monitor_mode {
        config.monitor.mode = match mode.as_str() {
            "full_mix" => gamesound_storage::config::MonitorMode::FullMix,
            _ => gamesound_storage::config::MonitorMode::SfxOnly,
        };
        changes.push(format!("monitor_mode={}", mode));
    }
    if let Some(enabled) = params.hotkeys_enabled {
        config.hotkeys.enabled = enabled;
        changes.push(format!("hotkeys_enabled={}", enabled));
    }
    if let Some(behavior) = params.close_behavior {
        config.desktop.close_behavior = match behavior.as_str() {
            "minimize_to_tray" => gamesound_storage::config::CloseBehavior::MinimizeToTray,
            "quit" => gamesound_storage::config::CloseBehavior::Quit,
            _ => gamesound_storage::config::CloseBehavior::Ask,
        };
        changes.push(format!("close_behavior={}", behavior));
    }

    state.store.save(&config).map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "update_settings", error = %e, "failed to save config");
        e.to_string()
    })?;

    tracing::info!(
        target: "gamesound_desktop::settings",
        changes = %changes.join(", "),
        "settings updated"
    );
    tracing::debug!(target: "gamesound_desktop::command", "success: update_settings");
    Ok(())
}

#[tauri::command]
pub fn export_config(state: State<AppState>) -> Result<String, String> {
    tracing::info!(target: "gamesound_desktop::command", "called: export_config");

    let config = state.store.load().map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "export_config", error = %e, "failed to load config");
        e.to_string()
    })?;
    let toml_str = toml::to_string_pretty(&config).map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "export_config", error = %e, "failed to serialise config");
        e.to_string()
    })?;

    tracing::info!(target: "gamesound_desktop::settings", "config exported");
    tracing::debug!(target: "gamesound_desktop::command", "success: export_config");
    Ok(toml_str)
}

#[tauri::command]
pub fn import_config(state: State<AppState>, toml_content: String) -> Result<(), String> {
    tracing::info!(target: "gamesound_desktop::command", "called: import_config");

    let config: gamesound_storage::config::AppConfig = toml::from_str(&toml_content).map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "import_config", error = %e, "invalid config format");
        format!("Invalid config format: {}", e)
    })?;
    state.store.save(&config).map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "import_config", error = %e, "failed to save config");
        e.to_string()
    })?;

    tracing::info!(target: "gamesound_desktop::settings", "config imported");
    tracing::debug!(target: "gamesound_desktop::command", "success: import_config");
    Ok(())
}

#[tauri::command]
pub fn reset_config(state: State<AppState>) -> Result<(), String> {
    tracing::info!(target: "gamesound_desktop::command", "called: reset_config");

    let default_config = gamesound_storage::config::AppConfig::default();
    state.store.save(&default_config).map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "reset_config", error = %e, "failed to save config");
        e.to_string()
    })?;

    tracing::warn!(target: "gamesound_desktop::settings", "config reset to defaults");
    tracing::debug!(target: "gamesound_desktop::command", "success: reset_config");
    Ok(())
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
    tracing::info!(target: "gamesound_desktop::command", "called: open_config_dir");

    let path = state.store.root().to_path_buf();
    tracing::info!(
        target: "gamesound_desktop::settings",
        path = %path.display(),
        "opening config directory"
    );

    open_in_file_manager(&path).map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "open_config_dir", error = %e, "failed to open config dir");
        e
    })?;

    tracing::debug!(target: "gamesound_desktop::command", "success: open_config_dir");
    Ok(())
}

#[tauri::command]
pub fn open_log_dir(state: State<AppState>) -> Result<(), String> {
    tracing::info!(target: "gamesound_desktop::command", "called: open_log_dir");

    let path = state.store.logs_path();
    if !path.exists() {
        std::fs::create_dir_all(&path).map_err(|e| {
            tracing::error!(target: "gamesound_desktop::command", command = "open_log_dir", error = %e, "failed to create log dir");
            e.to_string()
        })?;
    }

    tracing::info!(
        target: "gamesound_desktop::settings",
        path = %path.display(),
        "opening log directory"
    );

    open_in_file_manager(&path).map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "open_log_dir", error = %e, "failed to open log dir");
        e
    })?;

    tracing::debug!(target: "gamesound_desktop::command", "success: open_log_dir");
    Ok(())
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
    tracing::info!(target: "gamesound_desktop::command", "called: get_profile_info");

    let library = state.library.lock().unwrap();
    let profile = library.active_profile().map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "get_profile_info", error = %e, "failed to get active profile");
        e.to_string()
    })?;

    tracing::debug!(
        target: "gamesound_desktop::settings",
        profile_id = profile.id,
        name = %profile.name,
        "profile info returned"
    );
    tracing::debug!(target: "gamesound_desktop::command", "success: get_profile_info");

    Ok(ProfileInfo {
        id: profile.id,
        name: profile.name,
        description: profile.description,
        is_active: profile.is_active,
    })
}
