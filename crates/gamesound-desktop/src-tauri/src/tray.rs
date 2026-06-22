use crate::lifecycle::AppLifecycle;
use crate::state::AppState;
use gamesound_core::runtime::{RuntimeCommand, RuntimeStatus};
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};

/// Creates the system tray icon and menu.
pub fn create_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    tracing::info!(target: "gamesound_desktop::tray", "creating tray icon");

    let show_item = MenuItemBuilder::with_id("show", "Show Main Window").build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "Quit GameSound").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show_item)
        .item(&quit_item)
        .build()?;

    tracing::debug!(target: "gamesound_desktop::tray", "tray menu created");

    let tray_icon = match app.default_window_icon().cloned() {
        Some(icon) => icon,
        None => {
            tracing::error!(target: "gamesound_desktop::tray", "failed to load tray icon: default window icon missing");
            return Err(tauri::Error::from(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "default window icon missing",
            )));
        }
    };

    let _tray = TrayIconBuilder::with_id("gamesound-tray")
        .icon(tray_icon)
        .menu(&menu)
        .tooltip("GameSound Desktop")
        .on_menu_event(|app, event| {
            let lifecycle = app.state::<AppLifecycle>();
            match event.id().as_ref() {
                "show" => {
                    tracing::info!(target: "gamesound_desktop::tray", "menu clicked: show_main_window");
                    show_main_window(app);
                }
                "quit" => {
                    tracing::info!(target: "gamesound_desktop::tray", "menu clicked: quit_app");
                    lifecycle.set_quitting(true);
                    quit_app(app);
                }
                other => {
                    tracing::debug!(target: "gamesound_desktop::tray", menu_id = other, "unknown tray menu click");
                }
            }
        })
        .on_tray_icon_event(|tray, event| {
            #[cfg(target_os = "macos")]
            {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    tracing::info!(target: "gamesound_desktop::tray", "tray icon clicked (macOS left-click)");
                    show_main_window(tray.app_handle());
                }
            }
            #[cfg(target_os = "windows")]
            {
                if let TrayIconEvent::DoubleClick { .. } = event {
                    tracing::info!(target: "gamesound_desktop::tray", "tray icon double-clicked");
                    show_main_window(tray.app_handle());
                }
            }
        })
        .build(app)?;

    tracing::info!(target: "gamesound_desktop::tray", "tray icon created");
    Ok(())
}

/// Show and focus the main window.
fn show_main_window<R: Runtime>(app: &AppHandle<R>) {
    tracing::info!(target: "gamesound_desktop::tray", "show main window requested");

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        tracing::info!(target: "gamesound_desktop::window", "showing main window");
        tracing::info!(target: "gamesound_desktop::window", "focusing main window");
        #[cfg(target_os = "macos")]
        {
            let _ = window.unminimize();
            tracing::debug!(target: "gamesound_desktop::window", "window unminimized (macOS)");
        }
        tracing::info!(target: "gamesound_desktop::window", "window restored from tray");
    } else {
        tracing::warn!(target: "gamesound_desktop::tray", "main window not found");
    }
}

/// Gracefully quit: stop engine, release resources, exit.
pub fn quit_app<R: Runtime>(app: &AppHandle<R>) {
    tracing::info!(
        target: "gamesound_desktop::shutdown",
        "quit requested, source = tray"
    );

    let state = app.state::<AppState>();

    // Stop audio engine and wait for confirmation
    {
        let mut runtime_guard = state.runtime.lock().unwrap();
        if let Some(handle) = runtime_guard.take() {
            tracing::info!(target: "gamesound_desktop::shutdown", "sending shutdown to runtime");
            let (tx, rx) = std::sync::mpsc::channel();
            let _ = handle.commands.send(RuntimeCommand::Shutdown(tx));
            // Drop the lock so the runtime can process the command
            drop(runtime_guard);
            tracing::info!(target: "gamesound_desktop::shutdown", "waiting for runtime shutdown confirmation");
            match rx.recv() {
                Ok(()) => {
                    tracing::info!(target: "gamesound_desktop::shutdown", "runtime confirmed shutdown")
                }
                Err(_) => {
                    tracing::warn!(target: "gamesound_desktop::shutdown", "runtime shutdown confirmation channel closed")
                }
            }
        } else {
            tracing::debug!(target: "gamesound_desktop::shutdown", "audio engine not running, skip stop");
        }
    }
    {
        let mut status_guard = state.runtime_status.lock().unwrap();
        *status_guard = RuntimeStatus::Stopped;
    }

    tracing::info!(target: "gamesound_desktop::shutdown", "exiting application");
    app.exit(0);
}
