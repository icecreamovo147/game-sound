use crate::state::AppState;
use gamesound_core::{runtime::RuntimeCommand, Category, PlaybackMode, Sound};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundInfo {
    pub id: i64,
    pub name: String,
    pub file_path: String,
    pub category_id: Option<i64>,
    pub profile_id: Option<i64>,
    pub volume: f32,
    pub playback_mode: String,
    pub loop_enabled: bool,
    pub favorite: bool,
    pub tags: String,
    pub note: String,
    pub sort_order: i64,
    pub play_count: i64,
    pub last_played_at: Option<String>,
    pub hotkey: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryInfo {
    pub id: i64,
    pub name: String,
    pub profile_id: Option<i64>,
    pub sort_order: i64,
}

fn sound_to_info(sound: &Sound, hotkey: Option<String>) -> SoundInfo {
    SoundInfo {
        id: sound.id,
        name: sound.name.clone(),
        file_path: sound.file_path.clone(),
        category_id: sound.category_id,
        profile_id: sound.profile_id,
        volume: sound.volume,
        playback_mode: sound.playback_mode.as_str().to_string(),
        loop_enabled: sound.loop_enabled,
        favorite: sound.favorite,
        tags: sound.tags.clone(),
        note: sound.note.clone(),
        sort_order: sound.sort_order,
        play_count: sound.play_count,
        last_played_at: sound.last_played_at.clone(),
        hotkey,
    }
}

#[tauri::command]
pub fn list_sounds(
    state: State<AppState>,
    category: Option<i64>,
    query: Option<String>,
) -> Result<Vec<SoundInfo>, String> {
    tracing::info!(target: "gamesound_desktop::command", "called: list_sounds");

    let library = state.library.lock().unwrap();
    let active_profile = library.active_profile().ok();
    let profile_id = active_profile.as_ref().map(|p| p.id);
    let query = query.unwrap_or_default();

    let sounds: Vec<Sound> = if let Some(pid) = profile_id {
        library
            .sounds_in_profile(pid, category, &query)
            .map_err(|e| {
                tracing::error!(target: "gamesound_desktop::command", command = "list_sounds", error = %e, "failed to list sounds");
                e.to_string()
            })?
    } else {
        library
            .sounds(category, &query)
            .map_err(|e| {
                tracing::error!(target: "gamesound_desktop::command", command = "list_sounds", error = %e, "failed to list sounds");
                e.to_string()
            })?
    };

    let infos: Vec<SoundInfo> = sounds
        .iter()
        .map(|s| {
            let hotkey = library.hotkey(s.id).ok().flatten();
            sound_to_info(s, hotkey)
        })
        .collect();

    tracing::debug!(
        target: "gamesound_desktop::sounds",
        count = infos.len(),
        "sound list returned"
    );
    tracing::debug!(target: "gamesound_desktop::command", "success: list_sounds");
    Ok(infos)
}

#[tauri::command]
pub fn add_sound(
    state: State<AppState>,
    file_path: String,
    name: Option<String>,
    category_id: Option<i64>,
) -> Result<SoundInfo, String> {
    tracing::info!(target: "gamesound_desktop::command", "called: add_sound");

    let library = state.library.lock().unwrap();
    let active_profile = library.active_profile().map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "add_sound", error = %e, "failed to get active profile");
        e.to_string()
    })?;
    let profile_id = active_profile.id;

    let sound_name = name.unwrap_or_else(|| {
        std::path::Path::new(&file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Sound")
            .to_string()
    });

    let sound = Sound {
        id: 0,
        name: sound_name.clone(),
        file_path: file_path.clone(),
        category_id,
        profile_id: Some(profile_id),
        volume: 0.8,
        playback_mode: PlaybackMode::Overlay,
        loop_enabled: false,
        favorite: false,
        tags: String::new(),
        note: String::new(),
        sort_order: 0,
        play_count: 0,
        last_played_at: None,
    };

    let id = library.add_sound(sound).map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "add_sound", error = %e, "failed to add sound");
        e.to_string()
    })?;

    tracing::info!(
        target: "gamesound_desktop::sounds",
        sound_id = id,
        name = %sound_name,
        "sound added"
    );

    // Read back the inserted sound
    let sounds = library
        .sounds_in_profile(profile_id, None, "")
        .map_err(|e| {
            tracing::error!(target: "gamesound_desktop::command", command = "add_sound", error = %e, "failed to read back sound");
            e.to_string()
        })?;
    let inserted = sounds.iter().find(|s| s.id == id).ok_or_else(|| {
        tracing::error!(target: "gamesound_desktop::sounds", sound_id = id, "inserted sound not found on read-back");
        "Failed to read back inserted sound".to_string()
    })?;

    tracing::debug!(target: "gamesound_desktop::command", "success: add_sound");
    Ok(sound_to_info(inserted, None))
}

#[derive(Debug, Deserialize)]
pub struct UpdateSoundParams {
    pub id: i64,
    pub name: Option<String>,
    pub category_id: Option<i64>,
    pub volume: Option<f32>,
    pub playback_mode: Option<String>,
    pub loop_enabled: Option<bool>,
    pub favorite: Option<bool>,
    pub tags: Option<String>,
    pub note: Option<String>,
    pub sort_order: Option<i64>,
}

#[tauri::command]
pub fn update_sound(
    state: State<AppState>,
    params: UpdateSoundParams,
) -> Result<SoundInfo, String> {
    tracing::info!(target: "gamesound_desktop::command", "called: update_sound");

    let library = state.library.lock().unwrap();

    // Get existing sound
    let active_profile = library.active_profile().map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "update_sound", error = %e, "failed to get active profile");
        e.to_string()
    })?;
    let all_sounds = library
        .sounds_in_profile(active_profile.id, None, "")
        .map_err(|e| {
            tracing::error!(target: "gamesound_desktop::command", command = "update_sound", error = %e, "failed to list sounds");
            e.to_string()
        })?;
    let existing = all_sounds.iter().find(|s| s.id == params.id).ok_or_else(|| {
        tracing::warn!(target: "gamesound_desktop::sounds", sound_id = params.id, "sound not found for update");
        "Sound not found".to_string()
    })?;

    let mode = match params.playback_mode.as_deref() {
        Some("interrupt") => PlaybackMode::Interrupt,
        Some("queue") => PlaybackMode::Queue,
        Some("exclusive") => PlaybackMode::Exclusive,
        _ => existing.playback_mode,
    };

    let updated = Sound {
        id: params.id,
        name: params.name.unwrap_or_else(|| existing.name.clone()),
        category_id: params.category_id.or(existing.category_id),
        volume: params.volume.unwrap_or(existing.volume),
        playback_mode: mode,
        loop_enabled: params.loop_enabled.unwrap_or(existing.loop_enabled),
        favorite: params.favorite.unwrap_or(existing.favorite),
        tags: params.tags.unwrap_or_else(|| existing.tags.clone()),
        note: params.note.unwrap_or_else(|| existing.note.clone()),
        sort_order: params.sort_order.unwrap_or(existing.sort_order),
        file_path: existing.file_path.clone(),
        profile_id: existing.profile_id,
        play_count: existing.play_count,
        last_played_at: existing.last_played_at.clone(),
    };

    library.update_sound(&updated).map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "update_sound", error = %e, "failed to update sound");
        e.to_string()
    })?;

    tracing::info!(
        target: "gamesound_desktop::sounds",
        sound_id = params.id,
        name = %updated.name,
        "sound updated"
    );

    let hotkey = library.hotkey(params.id).ok().flatten();
    tracing::debug!(target: "gamesound_desktop::command", "success: update_sound");
    Ok(sound_to_info(&updated, hotkey))
}

#[tauri::command]
pub fn delete_sound(state: State<AppState>, id: i64) -> Result<(), String> {
    tracing::info!(target: "gamesound_desktop::command", "called: delete_sound");

    let library = state.library.lock().unwrap();
    library.remove_sound(id).map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "delete_sound", sound_id = id, error = %e, "failed to delete sound");
        e.to_string()
    })?;

    tracing::info!(target: "gamesound_desktop::sounds", sound_id = id, "sound deleted");
    tracing::debug!(target: "gamesound_desktop::command", "success: delete_sound");
    Ok(())
}

#[tauri::command]
pub fn play_sound(state: State<AppState>, id: i64) -> Result<(), String> {
    tracing::info!(target: "gamesound_desktop::command", "called: play_sound");

    let library = state.library.lock().unwrap();
    let active_profile = library.active_profile().map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "play_sound", error = %e, "failed to get active profile");
        e.to_string()
    })?;
    let all_sounds = library
        .sounds_in_profile(active_profile.id, None, "")
        .map_err(|e| {
            tracing::error!(target: "gamesound_desktop::command", command = "play_sound", error = %e, "failed to list sounds");
            e.to_string()
        })?;
    let sound = all_sounds
        .iter()
        .find(|s| s.id == id)
        .cloned()
        .ok_or_else(|| {
            tracing::warn!(target: "gamesound_desktop::sounds", sound_id = id, "sound not found");
            "Sound not found".to_string()
        })?;

    tracing::info!(
        target: "gamesound_desktop::sounds",
        sound_id = id,
        name = %sound.name,
        playback_mode = %sound.playback_mode.as_str(),
        "playing sound"
    );

    let runtime = state.runtime.lock().unwrap();
    if let Some(handle) = runtime.as_ref() {
        let _ = handle
            .commands
            .send(RuntimeCommand::PlaySound(sound.clone()));
        drop(runtime);
        drop(library);
        // Record play in a separate lock scope
        let lib = state.library.lock().unwrap();
        let _ = lib.record_play(id, "gui");
        tracing::debug!(target: "gamesound_desktop::sounds", sound_id = id, "play recorded");
        tracing::debug!(target: "gamesound_desktop::command", "success: play_sound");
        Ok(())
    } else {
        tracing::warn!(target: "gamesound_desktop::sounds", sound_id = id, "audio engine not running, cannot play");
        Err("Audio engine is not running. Please start it first.".into())
    }
}

#[tauri::command]
pub fn stop_sound(state: State<AppState>, id: i64) -> Result<(), String> {
    tracing::info!(target: "gamesound_desktop::command", "called: stop_sound");

    let runtime = state.runtime.lock().unwrap();
    if let Some(handle) = runtime.as_ref() {
        let _ = handle.commands.send(RuntimeCommand::StopSound(id));
        tracing::info!(target: "gamesound_desktop::sounds", sound_id = id, "stop sound requested");
        tracing::debug!(target: "gamesound_desktop::command", "success: stop_sound");
        Ok(())
    } else {
        tracing::warn!(target: "gamesound_desktop::sounds", sound_id = id, "audio engine not running");
        Err("Audio engine is not running.".into())
    }
}

#[tauri::command]
pub fn stop_all_sounds(state: State<AppState>) -> Result<(), String> {
    tracing::info!(target: "gamesound_desktop::command", "called: stop_all_sounds");

    let runtime = state.runtime.lock().unwrap();
    if let Some(handle) = runtime.as_ref() {
        let _ = handle.commands.send(RuntimeCommand::StopAll);
        tracing::info!(target: "gamesound_desktop::sounds", "stop all sounds requested");
        tracing::debug!(target: "gamesound_desktop::command", "success: stop_all_sounds");
        Ok(())
    } else {
        tracing::warn!(target: "gamesound_desktop::sounds", "audio engine not running");
        Err("Audio engine is not running.".into())
    }
}

#[tauri::command]
pub fn list_categories(state: State<AppState>) -> Result<Vec<CategoryInfo>, String> {
    tracing::info!(target: "gamesound_desktop::command", "called: list_categories");

    let library = state.library.lock().unwrap();
    let active_profile = library.active_profile().ok();
    let profile_id = active_profile.as_ref().map(|p| p.id);

    let categories: Vec<Category> = if let Some(pid) = profile_id {
        library.categories_in_profile(pid).map_err(|e| {
            tracing::error!(target: "gamesound_desktop::command", command = "list_categories", error = %e, "failed to list categories");
            e.to_string()
        })?
    } else {
        library.categories().map_err(|e| {
            tracing::error!(target: "gamesound_desktop::command", command = "list_categories", error = %e, "failed to list categories");
            e.to_string()
        })?
    };

    tracing::debug!(target: "gamesound_desktop::command", "success: list_categories");
    Ok(categories
        .into_iter()
        .map(|c| CategoryInfo {
            id: c.id,
            name: c.name,
            profile_id: c.profile_id,
            sort_order: c.sort_order,
        })
        .collect())
}

#[tauri::command]
pub fn add_category(
    state: State<AppState>,
    name: String,
    profile_id: Option<i64>,
) -> Result<CategoryInfo, String> {
    tracing::info!(target: "gamesound_desktop::command", "called: add_category");

    let library = state.library.lock().unwrap();
    let active_profile = library.active_profile().map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "add_category", error = %e, "failed to get active profile");
        e.to_string()
    })?;
    let pid = profile_id.or(Some(active_profile.id));

    let id = library.add_category(&name, pid).map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "add_category", error = %e, "failed to add category");
        e.to_string()
    })?;

    tracing::info!(target: "gamesound_desktop::sounds", category_id = id, name = %name, "category added");
    tracing::debug!(target: "gamesound_desktop::command", "success: add_category");
    Ok(CategoryInfo {
        id,
        name,
        profile_id: pid,
        sort_order: 0,
    })
}

#[tauri::command]
pub fn update_category(state: State<AppState>, id: i64, name: String) -> Result<(), String> {
    tracing::info!(target: "gamesound_desktop::command", "called: update_category");

    let library = state.library.lock().unwrap();
    library.rename_category(id, &name).map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "update_category", category_id = id, error = %e, "failed to rename category");
        e.to_string()
    })?;

    tracing::info!(target: "gamesound_desktop::sounds", category_id = id, name = %name, "category renamed");
    tracing::debug!(target: "gamesound_desktop::command", "success: update_category");
    Ok(())
}

#[tauri::command]
pub fn delete_category(state: State<AppState>, id: i64) -> Result<(), String> {
    tracing::info!(target: "gamesound_desktop::command", "called: delete_category");

    let library = state.library.lock().unwrap();
    library.remove_category(id).map_err(|e| {
        tracing::error!(target: "gamesound_desktop::command", command = "delete_category", category_id = id, error = %e, "failed to delete category");
        e.to_string()
    })?;

    tracing::info!(target: "gamesound_desktop::sounds", category_id = id, "category deleted");
    tracing::debug!(target: "gamesound_desktop::command", "success: delete_category");
    Ok(())
}
