use crate::state::AppState;
use gamesound_core::runtime::RuntimeEvent;
use std::time::Duration;
use tauri::{Emitter, Manager};

/// Spawns a background task that polls the runtime event channel and forwards
/// events to the Tauri frontend as window events.
pub fn spawn_event_forwarder(app: tauri::AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(50));

        let state = match app.try_state::<AppState>() {
            Some(s) => s,
            None => break,
        };

        let runtime_guard = state.runtime.lock().unwrap();
        let Some(handle) = runtime_guard.as_ref() else {
            continue;
        };

        // Drain all pending events
        while let Ok(event) = handle.events.try_recv() {
            let event_type = event_type_str(&event);
            let payload = serialize_event(&event);
            let _ = app.emit("runtime-event", payload);

            match &event {
                RuntimeEvent::Error(msg) => {
                    tracing::error!(
                        target: "gamesound_desktop::event",
                        event = event_type,
                        error = %msg,
                        "emitting error_occurred"
                    );
                }
                RuntimeEvent::Warning(msg) => {
                    tracing::warn!(
                        target: "gamesound_desktop::event",
                        event = event_type,
                        warning = %msg,
                        "emitting warning"
                    );
                }
                RuntimeEvent::Status(status) => {
                    tracing::info!(
                        target: "gamesound_desktop::event",
                        event = event_type,
                        status = format!("{:?}", status),
                        "emitting runtime_status_changed"
                    );
                }
                RuntimeEvent::SoundStarted(id) => {
                    tracing::info!(
                        target: "gamesound_desktop::event",
                        event = event_type,
                        sound_id = id,
                        "emitting sound_started"
                    );
                }
                RuntimeEvent::SoundStopped(id) => {
                    tracing::info!(
                        target: "gamesound_desktop::event",
                        event = event_type,
                        sound_id = id,
                        "emitting sound_stopped"
                    );
                }
                RuntimeEvent::HotkeysRegistered(count) => {
                    tracing::info!(
                        target: "gamesound_desktop::event",
                        event = event_type,
                        count = count,
                        "emitting hotkeys_registered"
                    );
                }
                RuntimeEvent::HotkeysSuspended => {
                    tracing::info!(
                        target: "gamesound_desktop::event",
                        event = event_type,
                        "emitting hotkeys_suspended"
                    );
                }
                RuntimeEvent::HotkeyCaptured(shortcut) => {
                    tracing::info!(
                        target: "gamesound_desktop::event",
                        event = event_type,
                        shortcut = shortcut,
                        "emitting hotkey_captured"
                    );
                }
                RuntimeEvent::SwitchProfileRequested => {
                    tracing::info!(
                        target: "gamesound_desktop::event",
                        event = event_type,
                        "emitting switch_profile_requested"
                    );
                }
                RuntimeEvent::Levels(_) => {
                    // Level events fire at ~20 Hz; log at trace level only
                    tracing::trace!(
                        target: "gamesound_desktop::event",
                        event = event_type,
                        "emitting levels"
                    );
                }
            }
        }
    });
}

fn event_type_str(event: &RuntimeEvent) -> &'static str {
    match event {
        RuntimeEvent::SoundStarted(_) => "SoundStarted",
        RuntimeEvent::SoundStopped(_) => "SoundStopped",
        RuntimeEvent::HotkeysSuspended => "HotkeysSuspended",
        RuntimeEvent::HotkeysRegistered(_) => "HotkeysRegistered",
        RuntimeEvent::HotkeyCaptured(_) => "HotkeyCaptured",
        RuntimeEvent::Levels(_) => "Levels",
        RuntimeEvent::Status(_) => "Status",
        RuntimeEvent::Error(_) => "Error",
        RuntimeEvent::Warning(_) => "Warning",
        RuntimeEvent::SwitchProfileRequested => "SwitchProfileRequested",
    }
}

fn serialize_event(event: &RuntimeEvent) -> serde_json::Value {
    match event {
        RuntimeEvent::SoundStarted(id) => serde_json::json!({
            "type": "SoundStarted",
            "data": { "id": id }
        }),
        RuntimeEvent::SoundStopped(id) => serde_json::json!({
            "type": "SoundStopped",
            "data": { "id": id }
        }),
        RuntimeEvent::HotkeysSuspended => serde_json::json!({
            "type": "HotkeysSuspended",
            "data": null
        }),
        RuntimeEvent::HotkeysRegistered(count) => serde_json::json!({
            "type": "HotkeysRegistered",
            "data": { "count": count }
        }),
        RuntimeEvent::HotkeyCaptured(shortcut) => serde_json::json!({
            "type": "HotkeyCaptured",
            "data": { "shortcut": shortcut }
        }),
        RuntimeEvent::Levels(levels) => serde_json::json!({
            "type": "Levels",
            "data": {
                "mic": levels.mic,
                "output": levels.output,
                "monitor": levels.monitor
            }
        }),
        RuntimeEvent::Status(status) => serde_json::json!({
            "type": "Status",
            "data": { "status": format!("{:?}", status) }
        }),
        RuntimeEvent::Error(message) => serde_json::json!({
            "type": "Error",
            "data": { "message": message }
        }),
        RuntimeEvent::Warning(message) => serde_json::json!({
            "type": "Warning",
            "data": { "message": message }
        }),
        RuntimeEvent::SwitchProfileRequested => serde_json::json!({
            "type": "SwitchProfileRequested",
            "data": null
        }),
    }
}
