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
    let _config = state.store.load().map_err(|e| e.to_string())?;

    let inputs = input_devices()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|d| DeviceInfo {
            id: d.id.clone(),
            name: d.name.clone(),
            is_virtual: d.is_virtual,
            device_type: "input".into(),
        })
        .collect();

    let outputs = output_devices()
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|d| DeviceInfo {
            id: d.id.clone(),
            name: d.name.clone(),
            is_virtual: d.is_virtual,
            device_type: "output".into(),
        })
        .collect();

    Ok(DeviceList { inputs, outputs })
}

#[tauri::command]
pub fn refresh_audio_devices(state: State<AppState>) -> Result<DeviceList, String> {
    list_audio_devices(state)
}

#[tauri::command]
pub fn set_mic_device(state: State<AppState>, device_name: String) -> Result<(), String> {
    let mut config = state.store.load().map_err(|e| e.to_string())?;
    config.audio.devices.mic = Some(device_name);
    state.store.save(&config).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn set_virtual_output_device(
    state: State<AppState>,
    device_name: String,
) -> Result<(), String> {
    let mut config = state.store.load().map_err(|e| e.to_string())?;
    config.audio.devices.output = Some(device_name);
    state.store.save(&config).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn set_monitor_device(state: State<AppState>, device_name: String) -> Result<(), String> {
    let mut config = state.store.load().map_err(|e| e.to_string())?;
    config.audio.devices.monitor = Some(device_name);
    state.store.save(&config).map_err(|e| e.to_string())?;
    Ok(())
}
