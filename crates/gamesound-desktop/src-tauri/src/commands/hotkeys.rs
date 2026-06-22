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
    tracing::info!(target: "gamesound_desktop::command", "called: list_hotkeys");

    let library = state.library.lock().unwrap();
    let active_profile = library.active_profile().ok();
    let profile_id = active_profile.as_ref().map(|p| p.id);

    let sounds = if let Some(pid) = profile_id {
        library.sounds_in_profile(pid, None, "").map_err(|e| {
            tracing::error!(target: "gamesound_desktop::command", command = "list_hotkeys", error = %e, "failed to list sounds");
            e.to_string()
        })?
    } else {
        library.sounds(None, "").map_err(|e| {
            tracing::error!(target: "gamesound_desktop::command", command = "list_hotkeys", error = %e, "failed to list sounds");
            e.to_string()
        })?
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

    tracing::debug!(
        target: "gamesound_desktop::hotkeys",
        binding_count = bindings.len(),
        "hotkey bindings listed"
    );
    tracing::debug!(target: "gamesound_desktop::command", "success: list_hotkeys");
    Ok(bindings)
}

#[tauri::command]
pub fn bind_hotkey(state: State<AppState>, sound_id: i64, hotkey: String) -> Result<(), String> {
    tracing::info!(target: "gamesound_desktop::command", "called: bind_hotkey");
    tracing::info!(
        target: "gamesound_desktop::hotkeys",
        sound_id = sound_id,
        hotkey = %hotkey,
        "binding hotkey"
    );

    let library = state.library.lock().unwrap();

    // Check for conflicts with system control hotkeys
    let config = state.store.load().map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "bind_hotkey", error = %e, "failed to load config");
        e.to_string()
    })?;
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
        tracing::warn!(
            target: "gamesound_desktop::hotkeys",
            hotkey = %hotkey,
            "hotkey conflicts with system control"
        );
        return Err(format!(
            "Hotkey '{}' is reserved for system controls",
            hotkey
        ));
    }

    library.set_hotkey(sound_id, &hotkey).map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "bind_hotkey", sound_id = sound_id, error = %e, "failed to set hotkey in database");
        e.to_string()
    })?;

    // Re-register hotkeys with the runtime if running
    let runtime = state.runtime.lock().unwrap();
    if let Some(handle) = runtime.as_ref() {
        let count = state.configure_hotkeys(handle).unwrap_or(0);
        tracing::info!(
            target: "gamesound_desktop::hotkeys",
            registered_count = count,
            "hotkeys re-registered after bind"
        );
    }

    tracing::info!(
        target: "gamesound_desktop::hotkeys",
        sound_id = sound_id,
        hotkey = %hotkey,
        "hotkey bound"
    );
    tracing::debug!(target: "gamesound_desktop::command", "success: bind_hotkey");
    Ok(())
}

#[tauri::command]
pub fn unbind_hotkey(state: State<AppState>, sound_id: i64) -> Result<(), String> {
    tracing::info!(target: "gamesound_desktop::command", "called: unbind_hotkey");

    let library = state.library.lock().unwrap();
    library.clear_hotkey(sound_id).map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "unbind_hotkey", sound_id = sound_id, error = %e, "failed to clear hotkey");
        e.to_string()
    })?;

    tracing::info!(
        target: "gamesound_desktop::hotkeys",
        sound_id = sound_id,
        "hotkey unbound"
    );

    // Re-register hotkeys
    let runtime = state.runtime.lock().unwrap();
    if let Some(handle) = runtime.as_ref() {
        let count = state.configure_hotkeys(handle).unwrap_or(0);
        tracing::info!(
            target: "gamesound_desktop::hotkeys",
            registered_count = count,
            "hotkeys re-registered after unbind"
        );
    }

    tracing::debug!(target: "gamesound_desktop::command", "success: unbind_hotkey");
    Ok(())
}

#[tauri::command]
pub fn enable_hotkeys(state: State<AppState>) -> Result<(), String> {
    tracing::info!(target: "gamesound_desktop::command", "called: enable_hotkeys");

    let mut config = state.store.load().map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "enable_hotkeys", error = %e, "failed to load config");
        e.to_string()
    })?;
    config.hotkeys.enabled = true;
    state.store.save(&config).map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "enable_hotkeys", error = %e, "failed to save config");
        e.to_string()
    })?;

    tracing::info!(
        target: "gamesound_desktop::hotkeys",
        "global hotkeys enabled"
    );

    let runtime = state.runtime.lock().unwrap();
    if let Some(handle) = runtime.as_ref() {
        let count = state.configure_hotkeys(handle).unwrap_or(0);
        tracing::info!(
            target: "gamesound_desktop::hotkeys",
            registered_count = count,
            "hotkeys registered after enable"
        );
    }

    tracing::debug!(target: "gamesound_desktop::command", "success: enable_hotkeys");
    Ok(())
}

#[tauri::command]
pub fn disable_hotkeys(state: State<AppState>) -> Result<(), String> {
    tracing::info!(target: "gamesound_desktop::command", "called: disable_hotkeys");

    let mut config = state.store.load().map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "disable_hotkeys", error = %e, "failed to load config");
        e.to_string()
    })?;
    config.hotkeys.enabled = false;
    state.store.save(&config).map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "disable_hotkeys", error = %e, "failed to save config");
        e.to_string()
    })?;

    tracing::info!(
        target: "gamesound_desktop::hotkeys",
        "global hotkeys disabled"
    );

    let runtime = state.runtime.lock().unwrap();
    if let Some(handle) = runtime.as_ref() {
        let _ = handle.commands.send(RuntimeCommand::SuspendHotkeys);
    }

    tracing::debug!(target: "gamesound_desktop::command", "success: disable_hotkeys");
    Ok(())
}

#[tauri::command]
pub fn reregister_hotkeys(state: State<AppState>) -> Result<usize, String> {
    tracing::info!(target: "gamesound_desktop::command", "called: reregister_hotkeys");

    let config = state.store.load().map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "reregister_hotkeys", error = %e, "failed to load config");
        e.to_string()
    })?;
    if !config.hotkeys.enabled {
        tracing::warn!(target: "gamesound_desktop::hotkeys", "hotkeys are disabled, cannot re-register");
        return Err("Hotkeys are currently disabled. Enable them first.".into());
    }

    let runtime = state.runtime.lock().unwrap();
    if let Some(handle) = runtime.as_ref() {
        let count = state.configure_hotkeys(handle).map_err(|e| {
            tracing::error!(target: "gamesound_desktop::command", command = "reregister_hotkeys", error = %e, "failed to configure hotkeys");
            e.to_string()
        })?;
        tracing::info!(
            target: "gamesound_desktop::hotkeys",
            registered_count = count,
            "hotkeys re-registered"
        );
        tracing::debug!(target: "gamesound_desktop::command", "success: reregister_hotkeys");
        Ok(count)
    } else {
        tracing::warn!(target: "gamesound_desktop::hotkeys", "audio engine not running, cannot re-register hotkeys");
        Err("Audio engine is not running.".into())
    }
}

/// Suspends current hotkeys and enters capture mode.
/// The next captured hotkey will be sent via the HotkeyCaptured runtime event.
#[tauri::command]
pub fn start_hotkey_capture(state: State<AppState>) -> Result<(), String> {
    tracing::info!(target: "gamesound_desktop::command", "called: start_hotkey_capture");

    let runtime = state.runtime.lock().unwrap();
    if let Some(handle) = runtime.as_ref() {
        let _ = handle.commands.send(RuntimeCommand::SuspendHotkeys);
        tracing::info!(
            target: "gamesound_desktop::hotkeys",
            "hotkey capture mode started"
        );
        tracing::debug!(target: "gamesound_desktop::command", "success: start_hotkey_capture");
        Ok(())
    } else {
        tracing::warn!(target: "gamesound_desktop::hotkeys", "audio engine not running, cannot start capture");
        Err("Audio engine is not running.".into())
    }
}
