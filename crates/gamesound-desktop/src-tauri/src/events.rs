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
            let payload = serialize_event(&event);
            let _ = app.emit("runtime-event", payload);
        }
    });
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
