use crate::state::AppState;
use gamesound_core::device::{input_devices, output_devices};
use serde::Serialize;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub is_virtual: bool,
    pub device_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceList {
    pub inputs: Vec<DeviceInfo>,
    pub outputs: Vec<DeviceInfo>,
}

#[tauri::command]
pub fn list_audio_devices(state: State<AppState>) -> Result<DeviceList, String> {
    tracing::info!(target: "gamesound_desktop::command", "called: list_audio_devices");
    tracing::info!(target: "gamesound_desktop::devices", "enumerating audio devices");

    let _config = state.store.load().map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "list_audio_devices", error = %e, "failed to load config");
        e.to_string()
    })?;

    let inputs = input_devices()
        .map_err(|e| {
            tracing::error!(target: "gamesound_desktop::devices", error = %e, "input device enumeration failed");
            e.to_string()
        })?
        .into_iter()
        .map(|d| DeviceInfo {
            id: d.id.clone(),
            name: d.name.clone(),
            is_virtual: d.is_virtual,
            device_type: "input".into(),
        })
        .collect::<Vec<_>>();

    let outputs = output_devices()
        .map_err(|e| {
            tracing::error!(target: "gamesound_desktop::devices", error = %e, "output device enumeration failed");
            e.to_string()
        })?
        .into_iter()
        .map(|d| DeviceInfo {
            id: d.id.clone(),
            name: d.name.clone(),
            is_virtual: d.is_virtual,
            device_type: "output".into(),
        })
        .collect::<Vec<_>>();

    tracing::info!(
        target: "gamesound_desktop::devices",
        input_count = inputs.len(),
        output_count = outputs.len(),
        "audio devices enumerated"
    );
    tracing::debug!(target: "gamesound_desktop::command", "success: list_audio_devices");

    Ok(DeviceList { inputs, outputs })
}

#[tauri::command]
pub fn refresh_audio_devices(state: State<AppState>) -> Result<DeviceList, String> {
    tracing::info!(target: "gamesound_desktop::command", "called: refresh_audio_devices");
    tracing::info!(target: "gamesound_desktop::devices", "refreshing audio devices");

    let result = list_audio_devices(state);
    match &result {
        Ok(_) => tracing::info!(target: "gamesound_desktop::devices", "audio devices refreshed"),
        Err(e) => {
            tracing::error!(target: "gamesound_desktop::devices", error = %e, "audio device refresh failed")
        }
    }
    result
}

#[tauri::command]
pub fn set_mic_device(state: State<AppState>, device_name: String) -> Result<(), String> {
    tracing::info!(target: "gamesound_desktop::command", "called: set_mic_device");
    tracing::info!(target: "gamesound_desktop::devices", mic_device = %device_name, "setting mic device");

    let mut config = state.store.load().map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "set_mic_device", error = %e, "failed to load config");
        e.to_string()
    })?;
    config.audio.devices.mic = Some(device_name);
    state.store.save(&config).map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "set_mic_device", error = %e, "failed to save config");
        e.to_string()
    })?;

    tracing::debug!(target: "gamesound_desktop::command", "success: set_mic_device");
    Ok(())
}

#[tauri::command]
pub fn set_virtual_output_device(
    state: State<AppState>,
    device_name: String,
) -> Result<(), String> {
    tracing::info!(target: "gamesound_desktop::command", "called: set_virtual_output_device");
    tracing::info!(target: "gamesound_desktop::devices", output_device = %device_name, "setting virtual output device");

    let mut config = state.store.load().map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "set_virtual_output_device", error = %e, "failed to load config");
        e.to_string()
    })?;
    config.audio.devices.output = Some(device_name);
    state.store.save(&config).map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "set_virtual_output_device", error = %e, "failed to save config");
        e.to_string()
    })?;

    tracing::debug!(target: "gamesound_desktop::command", "success: set_virtual_output_device");
    Ok(())
}

#[tauri::command]
pub fn set_monitor_device(state: State<AppState>, device_name: String) -> Result<(), String> {
    tracing::info!(target: "gamesound_desktop::command", "called: set_monitor_device");
    tracing::info!(target: "gamesound_desktop::devices", monitor_device = %device_name, "setting monitor device");

    let mut config = state.store.load().map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "set_monitor_device", error = %e, "failed to load config");
        e.to_string()
    })?;
    config.audio.devices.monitor = Some(device_name);
    state.store.save(&config).map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "set_monitor_device", error = %e, "failed to save config");
        e.to_string()
    })?;

    tracing::debug!(target: "gamesound_desktop::command", "success: set_monitor_device");
    Ok(())
}
