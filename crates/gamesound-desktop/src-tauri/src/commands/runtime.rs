use crate::state::AppState;
use gamesound_core::runtime::{RuntimeCommand, RuntimeStatus, VolumeTarget};
use serde::Serialize;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeStatusInfo {
    pub status: String,
    pub mic_device: Option<String>,
    pub output_device: Option<String>,
    pub monitor_device: Option<String>,
    pub hotkeys_enabled: bool,
    pub active_sounds: Vec<i64>,
}

#[tauri::command]
pub fn get_runtime_status(state: State<AppState>) -> Result<RuntimeStatusInfo, String> {
    let status = *state.runtime_status.lock().unwrap();
    let config = state.store.load().map_err(|e| e.to_string())?;

    Ok(RuntimeStatusInfo {
        status: match status {
            RuntimeStatus::Stopped => "Stopped".into(),
            RuntimeStatus::Running => "Running".into(),
            RuntimeStatus::Warning => "Warning".into(),
        },
        mic_device: config.audio.devices.mic.clone(),
        output_device: config.audio.devices.output.clone(),
        monitor_device: config.audio.devices.monitor.clone(),
        hotkeys_enabled: config.hotkeys.enabled,
        active_sounds: Vec::new(),
    })
}

#[tauri::command]
pub fn start_audio_engine(state: State<AppState>) -> Result<(), String> {
    let config = state.store.load().map_err(|e| e.to_string())?;
    let preferences = state.stream_preferences();

    let output_device = config.audio.devices.output.clone().ok_or_else(|| {
        "No virtual output device selected. Please configure it in Device Settings.".to_string()
    })?;

    let mut runtime_guard = state.runtime.lock().unwrap();
    if runtime_guard.is_some() {
        return Err("Audio engine is already running".into());
    }

    let handle = gamesound_core::runtime::spawn_runtime();

    let mic_device = config.audio.devices.mic.clone();
    let monitor_device = config.audio.devices.monitor.clone();
    let monitor_sfx_only = matches!(
        config.monitor.mode,
        gamesound_storage::config::MonitorMode::SfxOnly
    );

    let settings = *state.mixer_settings.lock().unwrap();
    let _ = handle.commands.send(RuntimeCommand::SetDucking {
        enabled: settings.ducking,
        ratio: settings.duck_ratio,
        attack_ms: settings.duck_attack_ms,
        release_ms: settings.duck_release_ms,
        release_delay_ms: settings.duck_release_delay_ms,
    });

    let _ = handle.commands.send(RuntimeCommand::Start {
        mic: mic_device,
        output: output_device,
        monitor: monitor_device,
        monitor_sfx_only,
        preferences,
    });

    // Apply initial volume settings
    let _ = handle.commands.send(RuntimeCommand::SetVolume {
        target: VolumeTarget::Mic,
        value: settings.mic_volume,
    });
    let _ = handle.commands.send(RuntimeCommand::SetVolume {
        target: VolumeTarget::Sfx,
        value: settings.sfx_volume,
    });
    let _ = handle.commands.send(RuntimeCommand::SetVolume {
        target: VolumeTarget::Monitor,
        value: settings.monitor_volume,
    });

    // Register hotkeys if enabled
    if config.hotkeys.enabled {
        let _ = state.configure_hotkeys(&handle);
    }

    // Update status and store handle
    *state.runtime_status.lock().unwrap() = RuntimeStatus::Running;
    *runtime_guard = Some(handle);

    Ok(())
}

#[tauri::command]
pub fn stop_audio_engine(state: State<AppState>) -> Result<(), String> {
    let mut runtime_guard = state.runtime.lock().unwrap();
    if let Some(handle) = runtime_guard.take() {
        let _ = handle.commands.send(RuntimeCommand::StopAudio);
    }
    *state.runtime_status.lock().unwrap() = RuntimeStatus::Stopped;
    Ok(())
}

#[tauri::command]
pub fn restart_audio_engine(state: State<AppState>) -> Result<(), String> {
    stop_audio_engine(state.clone())?;
    // Brief pause to let streams fully stop
    std::thread::sleep(std::time::Duration::from_millis(200));
    start_audio_engine(state)
}
