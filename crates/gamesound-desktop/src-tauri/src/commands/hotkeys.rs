use crate::state::AppState;
use gamesound_core::runtime::RuntimeCommand;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyBinding {
    pub sound_id: i64,
    pub sound_name: String,
    pub hotkey: String,
}

#[tauri::command]
pub fn list_hotkeys(state: State<AppState>) -> Result<Vec<HotkeyBinding>, String> {
    let library = state.library.lock().unwrap();
    let active_profile = library.active_profile().ok();
    let profile_id = active_profile.as_ref().map(|p| p.id);

    let sounds = if let Some(pid) = profile_id {
        library
            .sounds_in_profile(pid, None, "")
            .map_err(|e| e.to_string())?
    } else {
        library.sounds(None, "").map_err(|e| e.to_string())?
    };

    let mut bindings = Vec::new();
    for sound in &sounds {
        if let Ok(Some(hotkey)) = library.hotkey(sound.id) {
            bindings.push(HotkeyBinding {
                sound_id: sound.id,
                sound_name: sound.name.clone(),
                hotkey,
            });
        }
    }

    Ok(bindings)
}

#[tauri::command]
pub fn bind_hotkey(state: State<AppState>, sound_id: i64, hotkey: String) -> Result<(), String> {
    let library = state.library.lock().unwrap();

    // Check for conflicts
    let config = state.store.load().map_err(|e| e.to_string())?;
    let control_hotkeys = [
        &config.hotkeys.stop_all,
        &config.hotkeys.toggle_mic,
        &config.hotkeys.toggle_sfx,
        &config.hotkeys.toggle_monitor,
        &config.hotkeys.sfx_volume_up,
        &config.hotkeys.sfx_volume_down,
        &config.hotkeys.mic_volume_up,
        &config.hotkeys.mic_volume_down,
        &config.hotkeys.monitor_volume_up,
        &config.hotkeys.monitor_volume_down,
        &config.hotkeys.switch_profile,
    ];

    if control_hotkeys
        .iter()
        .any(|h| h.eq_ignore_ascii_case(&hotkey))
    {
        return Err(format!(
            "Hotkey '{}' is reserved for system controls",
            hotkey
        ));
    }

    library
        .set_hotkey(sound_id, &hotkey)
        .map_err(|e| e.to_string())?;

    // Re-register hotkeys with the runtime if running
    let runtime = state.runtime.lock().unwrap();
    if let Some(handle) = runtime.as_ref() {
        let _ = state.configure_hotkeys(handle);
    }

    Ok(())
}

#[tauri::command]
pub fn unbind_hotkey(state: State<AppState>, sound_id: i64) -> Result<(), String> {
    let library = state.library.lock().unwrap();
    library.clear_hotkey(sound_id).map_err(|e| e.to_string())?;

    // Re-register hotkeys
    let runtime = state.runtime.lock().unwrap();
    if let Some(handle) = runtime.as_ref() {
        let _ = state.configure_hotkeys(handle);
    }

    Ok(())
}

#[tauri::command]
pub fn enable_hotkeys(state: State<AppState>) -> Result<(), String> {
    let mut config = state.store.load().map_err(|e| e.to_string())?;
    config.hotkeys.enabled = true;
    state.store.save(&config).map_err(|e| e.to_string())?;

    let runtime = state.runtime.lock().unwrap();
    if let Some(handle) = runtime.as_ref() {
        let _ = state.configure_hotkeys(handle);
    }

    Ok(())
}

#[tauri::command]
pub fn disable_hotkeys(state: State<AppState>) -> Result<(), String> {
    let mut config = state.store.load().map_err(|e| e.to_string())?;
    config.hotkeys.enabled = false;
    state.store.save(&config).map_err(|e| e.to_string())?;

    let runtime = state.runtime.lock().unwrap();
    if let Some(handle) = runtime.as_ref() {
        let _ = handle.commands.send(RuntimeCommand::SuspendHotkeys);
    }

    Ok(())
}

#[tauri::command]
pub fn reregister_hotkeys(state: State<AppState>) -> Result<usize, String> {
    let config = state.store.load().map_err(|e| e.to_string())?;
    if !config.hotkeys.enabled {
        return Err("Hotkeys are currently disabled. Enable them first.".into());
    }

    let runtime = state.runtime.lock().unwrap();
    if let Some(handle) = runtime.as_ref() {
        state.configure_hotkeys(handle).map_err(|e| e.to_string())
    } else {
        Err("Audio engine is not running.".into())
    }
}

/// Suspends current hotkeys and enters capture mode.
/// The next captured hotkey will be sent via the HotkeyCaptured runtime event.
#[tauri::command]
pub fn start_hotkey_capture(state: State<AppState>) -> Result<(), String> {
    let runtime = state.runtime.lock().unwrap();
    if let Some(handle) = runtime.as_ref() {
        let _ = handle.commands.send(RuntimeCommand::SuspendHotkeys);
        Ok(())
    } else {
        Err("Audio engine is not running.".into())
    }
}
