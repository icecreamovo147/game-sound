use std::path::PathBuf;

/// Initialise the tracing subscriber for the desktop (Tauri) process.
///
/// Logs are always emitted to stderr (console). When a `log_dir` is provided
/// each run also appends to a rotating daily log file.
///
/// Safe to call multiple times — only the first call takes effect.
pub fn init_logging(log_dir: Option<&PathBuf>) {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("gamesound_desktop=debug,tauri=info"));

    let subscriber = fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(true)
        .compact();

    // File output is optional — the TUI process already writes file logs so
    // the desktop process focuses on console output for `pnpm tauri dev`.
    if let Some(dir) = log_dir {
        let _ = std::fs::create_dir_all(dir);
        let log_path = dir.join(format!(
            "gamesound-desktop-{}.log",
            chrono::Local::now().format("%Y-%m-%d")
        ));
        match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            Ok(file) => {
                // try_init silently ignores duplicate calls
                if subscriber
                    .with_writer(std::sync::Mutex::new(file))
                    .try_init()
                    .is_ok()
                {
                    tracing::info!(
                        target: "gamesound_desktop::logging",
                        log_path = %log_path.display(),
                        "logging initialised (console + file)"
                    );
                }
                return;
            }
            Err(e) => {
                eprintln!(
                    "[gamesound_desktop] cannot open log file {}: {e}",
                    log_path.display()
                );
            }
        }
    }

    // try_init silently ignores duplicate calls
    subscriber.try_init().ok();
}
