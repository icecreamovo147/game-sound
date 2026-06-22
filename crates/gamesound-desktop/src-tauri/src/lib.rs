use tauri::Manager;

mod commands;
mod events;
mod lifecycle;
pub mod logging;
mod state;
mod tray;

use lifecycle::AppLifecycle;
use state::AppState;

pub fn run() {
    tracing::info!(
        target: "gamesound_desktop::app",
        platform = %std::env::consts::OS,
        version = env!("CARGO_PKG_VERSION"),
        "GameSound starting"
    );

    let state = match AppState::new() {
        Ok(s) => {
            tracing::info!(target: "gamesound_desktop::app", "app state initialised");
            s
        }
        Err(e) => {
            tracing::error!(
                target: "gamesound_desktop::app",
                error = %e,
                "failed to initialise app state"
            );
            std::process::exit(1);
        }
    };

    // Re-init logging with the config directory available for file output.
    logging::init_logging(Some(&state.store.logs_path()));
    tracing::info!(
        target: "gamesound_desktop::app",
        config_dir = %state.store.root().display(),
        "config directory ready"
    );

    let app_lifecycle = AppLifecycle::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .manage(app_lifecycle)
        .setup(|app| {
            tracing::info!(target: "gamesound_desktop::tauri", "setup started");

            let handle = app.handle().clone();

            // Restore saved config and start event polling
            let app_state = app.state::<AppState>();
            match app_state.restore_config() {
                Ok(()) => tracing::debug!(target: "gamesound_desktop::tauri", "config restored"),
                Err(e) => tracing::warn!(target: "gamesound_desktop::tauri", error = %e, "config restore failed"),
            }
            let _ = app_state.restore_hotkeys();

            tracing::info!(target: "gamesound_desktop::tauri", "app state registered");

            // Create system tray
            match tray::create_tray(app.handle()) {
                Ok(()) => tracing::info!(target: "gamesound_desktop::tauri", "tray initialised"),
                Err(e) => tracing::error!(target: "gamesound_desktop::tauri", error = %e, "tray initialisation failed"),
            }

            // Intercept main window close event
            match app.get_webview_window("main") {
                Some(window) => {
                    tracing::info!(target: "gamesound_desktop::tauri", "main window found");

                    let window_clone = window.clone();
                    let handle_clone = app.handle().clone();
                    window.on_window_event(move |event| {
                        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                            // Always prevent the default close — we handle it ourselves
                            api.prevent_close();

                            let lifecycle = handle_clone.state::<AppLifecycle>();
                            let state = handle_clone.state::<AppState>();
                            let config = state.store.load().unwrap_or_default();
                            let behavior = match config.desktop.close_behavior {
                                gamesound_storage::config::CloseBehavior::Ask => "ask",
                                gamesound_storage::config::CloseBehavior::MinimizeToTray => {
                                    "minimize_to_tray"
                                }
                                gamesound_storage::config::CloseBehavior::Quit => "quit",
                            };
                            lifecycle::handle_close_request(&window_clone, &lifecycle, behavior);
                        }
                    });

                    tracing::info!(target: "gamesound_desktop::tauri", "window event handlers registered");
                }
                None => {
                    tracing::warn!(target: "gamesound_desktop::tauri", "main window missing");
                }
            }

            // Spawn background event-forwarding task
            events::spawn_event_forwarder(handle.clone());
            tracing::debug!(target: "gamesound_desktop::tauri", "event forwarder spawned");

            tracing::info!(target: "gamesound_desktop::tauri", "setup completed");
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
            commands::runtime::confirm_close_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running GameSound Desktop");
}
