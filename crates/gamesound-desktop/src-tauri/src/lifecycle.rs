use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tauri::Emitter;
use tauri::Manager;

/// Tracks whether the application is in an intentional quit flow.
/// When true, close-event handlers should NOT show a confirmation dialog.
pub struct AppLifecycle {
    pub is_quitting: AtomicBool,
    #[allow(dead_code)]
    pub close_dialog_open: Mutex<bool>,
}

impl AppLifecycle {
    pub fn new() -> Self {
        Self {
            is_quitting: AtomicBool::new(false),
            close_dialog_open: Mutex::new(false),
        }
    }

    pub fn set_quitting(&self, value: bool) {
        tracing::info!(
            target: "gamesound_desktop::lifecycle",
            quitting = value,
            "quit flag set"
        );
        self.is_quitting.store(value, Ordering::SeqCst);
    }

    pub fn is_quitting(&self) -> bool {
        self.is_quitting.load(Ordering::SeqCst)
    }
}

/// Platform-specific close-window handler.
/// On Windows: shows a native TaskDialog (or MessageBox fallback).
/// On macOS/Linux: hides to tray (macOS convention) or closes based on behavior.
#[cfg(target_os = "windows")]
pub fn handle_close_request(
    window: &tauri::WebviewWindow,
    lifecycle: &AppLifecycle,
    close_behavior: &str,
) {
    use crate::tray;
    use gamesound_storage::config::CloseBehavior;

    tracing::info!(
        target: "gamesound_desktop::window",
        behavior = close_behavior,
        "close requested"
    );

    if lifecycle.is_quitting() {
        tracing::debug!(target: "gamesound_desktop::window", "quit flow in progress, allowing close");
        return;
    }

    let behavior: CloseBehavior = match close_behavior {
        "ask" => CloseBehavior::Ask,
        "minimize_to_tray" => CloseBehavior::MinimizeToTray,
        "quit" => CloseBehavior::Quit,
        _ => CloseBehavior::Ask,
    };

    match behavior {
        CloseBehavior::Quit => {
            tracing::info!(target: "gamesound_desktop::window", "close behavior = quit");
            lifecycle.set_quitting(true);
            tray::quit_app(window.app_handle());
        }
        CloseBehavior::MinimizeToTray => {
            tracing::info!(target: "gamesound_desktop::window", "close behavior = minimize_to_tray");
            let _ = window.hide();
            tracing::info!(target: "gamesound_desktop::window", "main window hidden");
        }
        CloseBehavior::Ask => {
            tracing::info!(target: "gamesound_desktop::window", "close behavior = ask");
            // Prevent re-entrant dialog
            {
                let mut guard = lifecycle.close_dialog_open.lock().unwrap();
                if *guard {
                    tracing::warn!(target: "gamesound_desktop::window", "close dialog already open, ignoring");
                    return;
                }
                *guard = true;
            }

            tracing::info!(target: "gamesound_desktop::window", "showing native close confirmation dialog");
            // Read language preference for i18n
            let is_chinese = window
                .app_handle()
                .try_state::<crate::state::AppState>()
                .and_then(|s| s.store.load().ok())
                .map(|c| matches!(c.tui.language, gamesound_storage::config::Language::Chinese))
                .unwrap_or(true);
            // Show native Windows confirmation dialog
            let result = show_windows_close_dialog(is_chinese);

            {
                let mut guard = lifecycle.close_dialog_open.lock().unwrap();
                *guard = false;
            }

            match result {
                WindowsCloseChoice::Quit => {
                    tracing::info!(target: "gamesound_desktop::window", "close dialog result = quit");
                    lifecycle.set_quitting(true);
                    tray::quit_app(window.app_handle());
                }
                WindowsCloseChoice::MinimizeToTray => {
                    tracing::info!(target: "gamesound_desktop::window", "close dialog result = minimize_to_tray");
                    let _ = window.hide();
                    tracing::info!(target: "gamesound_desktop::window", "minimizing to tray");
                }
                WindowsCloseChoice::Cancel => {
                    tracing::info!(target: "gamesound_desktop::window", "close dialog result = cancel");
                    tracing::info!(target: "gamesound_desktop::window", "close cancelled");
                }
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn handle_close_request(
    window: &tauri::WebviewWindow,
    lifecycle: &AppLifecycle,
    close_behavior: &str,
) {
    use crate::tray;
    use gamesound_storage::config::CloseBehavior;

    tracing::info!(
        target: "gamesound_desktop::window",
        behavior = close_behavior,
        "close requested"
    );

    if lifecycle.is_quitting() {
        tracing::debug!(target: "gamesound_desktop::window", "quit flow in progress, allowing close");
        return;
    }

    let behavior: CloseBehavior = match close_behavior {
        "ask" => CloseBehavior::Ask,
        "minimize_to_tray" => CloseBehavior::MinimizeToTray,
        "quit" => CloseBehavior::Quit,
        _ => CloseBehavior::Ask,
    };

    match behavior {
        CloseBehavior::Quit => {
            tracing::info!(target: "gamesound_desktop::window", "close behavior = quit");
            lifecycle.set_quitting(true);
            tray::quit_app(window.app_handle());
        }
        CloseBehavior::MinimizeToTray => {
            tracing::info!(target: "gamesound_desktop::window", "close behavior = minimize_to_tray");
            let _ = window.hide();
            tracing::info!(target: "gamesound_desktop::window", "minimizing to tray");
        }
        CloseBehavior::Ask => {
            tracing::info!(target: "gamesound_desktop::window", "close behavior = ask");
            // Emit event to frontend so it can show a confirmation dialog.
            // The frontend calls confirm_close_window Tauri command with the user's choice.
            let handle = window.app_handle().clone();
            let _ = handle.emit("show-close-dialog", ());
            tracing::info!(target: "gamesound_desktop::window", "close-dialog event emitted to frontend");
        }
    }
}

// ── Windows-specific native dialog ──

#[cfg(target_os = "windows")]
#[derive(Debug, PartialEq)]
enum WindowsCloseChoice {
    Quit,
    MinimizeToTray,
    Cancel,
}

/// Show a native Windows close-confirmation dialog.
/// Tries TaskDialog first, falls back to MessageBoxW.
#[cfg(target_os = "windows")]
fn show_windows_close_dialog(is_chinese: bool) -> WindowsCloseChoice {
    // Try TaskDialogIndirect for custom button text
    if let Some(choice) = show_task_dialog() {
        return choice;
    }
    tracing::debug!(target: "gamesound_desktop::window", "TaskDialog unavailable, falling back to MessageBox");
    // Fallback: MessageBoxW with Yes/No/Cancel
    show_message_box_close_dialog()
}

#[cfg(target_os = "windows")]
fn show_task_dialog(is_chinese: bool) -> Option<WindowsCloseChoice> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    extern "system" {
        fn TaskDialog(
            hwnd: isize,
            hinstance: isize,
            psz_window_title: *const u16,
            psz_main_instruction: *const u16,
            psz_content: *const u16,
            dw_common_buttons: u32,
            psz_icon: isize,
            pn_button: *mut i32,
        ) -> i32;
    }

    const TDCBF_OK_BUTTON: u32 = 0x0001;
    const TDCBF_YES_BUTTON: u32 = 0x0002;
    const TDCBF_NO_BUTTON: u32 = 0x0004;
    const TDCBF_CANCEL_BUTTON: u32 = 0x0008;
    const IDYES: i32 = 6;
    const IDNO: i32 = 7;
    const IDCANCEL: i32 = 2;

    let (title_str, instruction_str, content_str) = if is_chinese {
        (
            "关闭 GameSound？",
            "关闭 GameSound？",
            "你想直接关闭软件，还是最小化到托盘继续在后台运行？",
        )
    } else {
        (
            "Close GameSound?",
            "Close GameSound?",
            "Do you want to quit, or minimize to tray to keep running in the background?",
        )
    };
    let title: Vec<u16> = OsStr::new(title_str)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let instruction: Vec<u16> = OsStr::new(instruction_str)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let content: Vec<u16> = OsStr::new(content_str)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let mut button: i32 = 0;
    let result = unsafe {
        TaskDialog(
            0, // hwnd parent
            0, // hinstance
            title.as_ptr(),
            instruction.as_ptr(),
            content.as_ptr(),
            TDCBF_YES_BUTTON | TDCBF_NO_BUTTON | TDCBF_CANCEL_BUTTON,
            0, // no custom icon
            &mut button,
        )
    };

    if result < 0 {
        // TaskDialog failed — caller should fallback to MessageBox
        return None;
    }

    match button {
        // Yes = Minimize to tray
        IDYES => Some(WindowsCloseChoice::MinimizeToTray),
        // No = Quit
        IDNO => Some(WindowsCloseChoice::Quit),
        // Cancel = Cancel
        IDCANCEL => Some(WindowsCloseChoice::Cancel),
        _ => Some(WindowsCloseChoice::Cancel),
    }
}

#[cfg(target_os = "windows")]
fn show_message_box_close_dialog(is_chinese: bool) -> WindowsCloseChoice {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    extern "system" {
        fn MessageBoxW(hwnd: isize, text: *const u16, caption: *const u16, utype: u32) -> i32;
    }

    const MB_YESNOCANCEL: u32 = 0x00000003;
    const MB_ICONQUESTION: u32 = 0x00000020;
    const IDYES: i32 = 6;
    const IDNO: i32 = 7;
    const IDCANCEL: i32 = 2;

    let (caption_str, text_str) = if is_chinese {
        (
            "关闭 GameSound？",
            "你想直接关闭软件 (否)，还是最小化到托盘继续在后台运行 (是)？",
        )
    } else {
        (
            "Close GameSound?",
            "Do you want to quit (No), or minimize to tray to keep running (Yes)?",
        )
    };
    let caption: Vec<u16> = OsStr::new(caption_str)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // Button mapping:
    //   Yes    = Minimize to tray
    //   No     = Quit
    //   Cancel = Cancel
    let text: Vec<u16> = OsStr::new(text_str)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let result = unsafe {
        MessageBoxW(
            0,
            text.as_ptr(),
            caption.as_ptr(),
            MB_YESNOCANCEL | MB_ICONQUESTION,
        )
    };

    match result {
        IDYES => WindowsCloseChoice::MinimizeToTray,
        IDNO => WindowsCloseChoice::Quit,
        _ => WindowsCloseChoice::Cancel,
    }
}
