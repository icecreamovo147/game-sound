use crate::state::AppState;
use gamesound_core::runtime::{RuntimeCommand, RuntimeStatus, VolumeTarget};
use serde::Serialize;
use tauri::Manager;
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
    tracing::info!(target: "gamesound_desktop::command", "called: get_runtime_status");

    let status = *state.runtime_status.lock().unwrap();
    let config = state.store.load().map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "get_runtime_status", error = %e, "failed to load config");
        e.to_string()
    })?;

    tracing::debug!(target: "gamesound_desktop::command", "success: get_runtime_status");
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
    tracing::info!(target: "gamesound_desktop::command", "called: start_audio_engine");

    let config = state.store.load().map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "start_audio_engine", error = %e, "failed to load config");
        e.to_string()
    })?;
    let preferences = state.stream_preferences();

    tracing::info!(
        target: "gamesound_desktop::audio",
        sample_rate = preferences.sample_rate,
        channels = preferences.channels,
        buffer_size = preferences.buffer_size,
        "audio engine starting"
    );

    let output_device = config.audio.devices.output.clone().ok_or_else(|| {
        tracing::error!(target: "gamesound_desktop::audio", "no virtual output device configured");
        "No virtual output device selected. Please configure it in Device Settings.".to_string()
    })?;

    tracing::info!(
        target: "gamesound_desktop::audio",
        output_device = %output_device,
        mic_device = config.audio.devices.mic.as_deref().unwrap_or("none"),
        monitor_device = config.audio.devices.monitor.as_deref().unwrap_or("none"),
        "audio device configuration"
    );

    let mut runtime_guard = state.runtime.lock().unwrap();
    if runtime_guard.is_some() {
        tracing::warn!(target: "gamesound_desktop::audio", "audio engine already running, refusing to start again");
        return Err("Audio engine is already running".into());
    }

    let handle = gamesound_core::runtime::spawn_runtime();
    tracing::info!(target: "gamesound_desktop::audio", "runtime spawned");

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
        tracing::info!(target: "gamesound_desktop::audio", "hotkeys enabled, registering");
        let _ = state.configure_hotkeys(&handle);
    } else {
        tracing::debug!(target: "gamesound_desktop::audio", "hotkeys disabled");
    }

    // Update status and store handle
    *state.runtime_status.lock().unwrap() = RuntimeStatus::Running;
    *runtime_guard = Some(handle);

    tracing::info!(target: "gamesound_desktop::audio", "audio engine started");
    tracing::debug!(target: "gamesound_desktop::command", "success: start_audio_engine");
    Ok(())
}

#[tauri::command]
pub fn stop_audio_engine(state: State<AppState>) -> Result<(), String> {
    tracing::info!(target: "gamesound_desktop::command", "called: stop_audio_engine");
    tracing::info!(target: "gamesound_desktop::audio", "audio engine stopping");

    let mut runtime_guard = state.runtime.lock().unwrap();
    if let Some(handle) = runtime_guard.take() {
        let _ = handle.commands.send(RuntimeCommand::StopAudio);
        tracing::info!(target: "gamesound_desktop::audio", "stop command sent");
    } else {
        tracing::debug!(target: "gamesound_desktop::audio", "audio engine not running");
    }
    *state.runtime_status.lock().unwrap() = RuntimeStatus::Stopped;

    tracing::info!(target: "gamesound_desktop::audio", "audio engine stopped");
    tracing::debug!(target: "gamesound_desktop::command", "success: stop_audio_engine");
    Ok(())
}

#[tauri::command]
pub fn restart_audio_engine(state: State<AppState>) -> Result<(), String> {
    tracing::info!(target: "gamesound_desktop::command", "called: restart_audio_engine");
    tracing::info!(target: "gamesound_desktop::audio", "audio engine restarting");

    stop_audio_engine(state.clone())?;
    // Brief pause to let streams fully stop
    std::thread::sleep(std::time::Duration::from_millis(200));
    let result = start_audio_engine(state);

    match &result {
        Ok(()) => tracing::info!(target: "gamesound_desktop::audio", "audio engine restarted"),
        Err(e) => {
            tracing::error!(target: "gamesound_desktop::audio", error = %e, "audio engine restart failed")
        }
    }
    tracing::debug!(target: "gamesound_desktop::command", "success: restart_audio_engine");
    result
}

#[tauri::command]
pub fn confirm_close_window(
    app: tauri::AppHandle,
    lifecycle: tauri::State<crate::lifecycle::AppLifecycle>,
    action: String,
) -> Result<(), String> {
    tracing::info!(
        target: "gamesound_desktop::command",
        action = %action,
        "called: confirm_close_window"
    );

    match action.as_str() {
        "quit" => {
            tracing::info!(target: "gamesound_desktop::window", "close dialog: quit");
            lifecycle.set_quitting(true);
            crate::tray::quit_app(&app);
        }
        "minimize_to_tray" => {
            tracing::info!(target: "gamesound_desktop::window", "close dialog: minimize_to_tray");
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
                tracing::info!(target: "gamesound_desktop::window", "window hidden to tray");
            }
        }
        "cancel" => {
            tracing::info!(target: "gamesound_desktop::window", "close dialog: cancel");
            // Do nothing — window stays open
        }
        other => {
            tracing::warn!(
                target: "gamesound_desktop::command",
                action = other,
                "unknown close action, defaulting to cancel"
            );
        }
    }

    Ok(())
}
