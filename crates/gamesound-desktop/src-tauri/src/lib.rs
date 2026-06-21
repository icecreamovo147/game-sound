use tauri::Manager;

mod commands;
mod events;
mod state;

use state::AppState;

pub fn run() {
    let state = AppState::new().expect("failed to initialise GameSound desktop state");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .setup(|app| {
            let handle = app.handle().clone();

            // Restore saved config and start event polling
            let app_state = app.state::<AppState>();
            let _ = app_state.restore_config();
            let _ = app_state.restore_hotkeys();

            // Spawn background event-forwarding task
            events::spawn_event_forwarder(handle.clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::runtime::get_runtime_status,
            commands::runtime::start_audio_engine,
            commands::runtime::stop_audio_engine,
            commands::runtime::restart_audio_engine,
            commands::sounds::list_sounds,
            commands::sounds::add_sound,
            commands::sounds::update_sound,
            commands::sounds::delete_sound,
            commands::sounds::play_sound,
            commands::sounds::stop_sound,
            commands::sounds::stop_all_sounds,
            commands::sounds::list_categories,
            commands::sounds::add_category,
            commands::sounds::update_category,
            commands::sounds::delete_category,
            commands::devices::list_audio_devices,
            commands::devices::refresh_audio_devices,
            commands::devices::set_mic_device,
            commands::devices::set_virtual_output_device,
            commands::devices::set_monitor_device,
            commands::mixer::get_mixer_settings,
            commands::mixer::update_mixer_settings,
            commands::mixer::set_mic_volume,
            commands::mixer::set_sfx_volume,
            commands::mixer::set_monitor_volume,
            commands::mixer::toggle_mic_mute,
            commands::mixer::toggle_sfx_mute,
            commands::mixer::toggle_monitor,
            commands::hotkeys::list_hotkeys,
            commands::hotkeys::bind_hotkey,
            commands::hotkeys::unbind_hotkey,
            commands::hotkeys::enable_hotkeys,
            commands::hotkeys::disable_hotkeys,
            commands::hotkeys::reregister_hotkeys,
            commands::hotkeys::start_hotkey_capture,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::export_config,
            commands::settings::import_config,
            commands::settings::reset_config,
            commands::settings::open_config_dir,
            commands::settings::open_log_dir,
            commands::settings::get_profile_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running GameSound Desktop");
}
