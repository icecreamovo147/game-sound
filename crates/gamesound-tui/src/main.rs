mod app;
mod ui;
use anyhow::{Context, Result};
use app::App;
use clap::{Parser, Subcommand};
use crossterm::{
    cursor::Show,
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use gamesound_core::device;
use gamesound_storage::{ConfigStore, Library};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

#[derive(Parser)]
#[command(
    name = "gamesound",
    about = "A terminal soundboard that mixes microphone and effects into a virtual audio device"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}
#[derive(Subcommand)]
enum Command {
    Doctor,
    ResetConfig,
    /// Import one file or recursively scan a directory.
    Import {
        path: std::path::PathBuf,
        /// Copy audio into GameSound's managed sounds directory instead of referencing the source path.
        #[arg(long)]
        copy: bool,
    },
    /// Export the active sound library as a portable JSON manifest.
    Export {
        file: std::path::PathBuf,
    },
    /// Save a point-in-time config/database backup.
    Backup,
}
fn main() -> Result<()> {
    let cli = Cli::parse();
    let store = ConfigStore::for_current_user()?;
    if matches!(&cli.command, Some(Command::Doctor)) {
        tracing_subscriber::fmt()
            .with_env_filter("warn")
            .with_writer(std::io::stderr)
            .init();
    } else {
        store.initialise()?;
        let log_level = store
            .load()
            .map(|config| config.app.log_level)
            .unwrap_or_else(|_| "warn".into());
        let log_file =
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(store.logs_path().join(format!(
                    "gamesound-{}.log",
                    chrono::Local::now().format("%Y-%m-%d")
                )))?;
        tracing_subscriber::fmt()
            .with_env_filter(log_level)
            .with_writer(log_file)
            .init();
    }
    match cli.command {
        Some(Command::Doctor) => doctor(&store),
        Some(Command::ResetConfig) => {
            store.save(&Default::default())?;
            println!("Reset {}", store.config_path().display());
            Ok(())
        }
        Some(Command::Import { path, copy }) => {
            let library = open_library_with_recovery(&store)?;
            let profile = library.active_profile()?.id;
            if path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
            {
                let count =
                    library.import_profile_json(&std::fs::read_to_string(&path)?, Some(profile))?;
                println!("Imported {count} sound(s) from manifest");
            } else {
                let ids = library.import_path_with_mode(
                    &path,
                    Some(profile),
                    copy.then(|| store.sounds_path()).as_deref(),
                )?;
                println!("Imported {} audio file(s)", ids.len());
            }
            Ok(())
        }
        Some(Command::Export { file }) => {
            let library = open_library_with_recovery(&store)?;
            std::fs::write(
                &file,
                library.export_profile_json(Some(library.active_profile()?.id))?,
            )?;
            println!("Exported {}", file.display());
            Ok(())
        }
        Some(Command::Backup) => {
            println!("Backup created at {}", store.backup()?.display());
            Ok(())
        }
        None => run(store),
    }
}
fn doctor(store: &ConfigStore) -> Result<()> {
    println!("GameSound doctor\n");
    // Doctor is intentionally read-only: it must work even before first launch
    // and in a sandbox where the regular data directory is not writable.
    let config = if store.config_path().is_file() {
        store.load().unwrap_or_default()
    } else {
        Default::default()
    };
    let inputs = match device::input_devices() {
        Ok(v) => {
            println!("Input devices:");
            for d in &v {
                println!(
                    "  {}{}",
                    d.name,
                    if d.is_virtual { " (virtual)" } else { "" }
                );
            }
            v
        }
        Err(e) => {
            println!("Input enumeration failed: {e}");
            Vec::new()
        }
    };
    let outputs = match device::output_devices() {
        Ok(v) => {
            println!("\nOutput devices:");
            for d in &v {
                println!(
                    "  {}{}",
                    d.name,
                    if d.is_virtual { " (virtual)" } else { "" }
                );
            }
            v
        }
        Err(e) => {
            println!("Output enumeration failed: {e}");
            Vec::new()
        }
    };
    let selected = |label: &str,
                    value: Option<&String>,
                    names: &[gamesound_core::device::AudioDevice]| match value {
        Some(name) if names.iter().any(|device| &device.id == name) => {
            println!("{label}: {name} [available]")
        }
        Some(name) => println!("{label}: {name} [MISSING - choose another device]"),
        None => println!("{label}: not configured"),
    };
    println!("\nConfigured devices:");
    selected("Mic", config.audio.devices.mic.as_ref(), &inputs);
    selected(
        "Virtual output",
        config.audio.devices.output.as_ref(),
        &outputs,
    );
    selected("Monitor", config.audio.devices.monitor.as_ref(), &outputs);
    if let Some(name) = config.audio.devices.output.as_ref() {
        if let Some(device) = outputs.iter().find(|device| &device.id == name) {
            if !device.is_virtual {
                println!("WARNING: output does not look like a virtual audio device.");
            }
        }
    }
    if store.db_path().is_file() {
        match Library::open(&store.db_path()) {
            Ok(library) => println!(
                "Database schema: v{}; active profile: {}",
                library.schema_version()?,
                library.active_profile()?.name
            ),
            Err(error) => println!("Database diagnostic unavailable: {error}"),
        }
    } else {
        println!("Database: not initialized yet (launch GameSound once to create it)");
    }
    println!("\nSelect a virtual output (BlackHole/VB-CABLE/Loopback) in GameSound, then choose its matching input in your voice app.");
    Ok(())
}
fn run(store: ConfigStore) -> Result<()> {
    let config = store.load().unwrap_or_else(|e| {
        eprintln!("{e}");
        Default::default()
    });
    let library = open_library_with_recovery(&store)?;
    enable_raw_mode()?;
    let _guard = TerminalGuard;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    event_loop(&mut terminal, App::new(library, store, config))
}

/// Preserve an unreadable SQLite database before creating a fresh, migrated
/// library. The user can recover it manually or restore a backup afterwards.
fn open_library_with_recovery(store: &ConfigStore) -> Result<Library> {
    store.initialise()?;
    match Library::open(&store.db_path()) {
        Ok(library) => Ok(library),
        Err(error) if store.db_path().exists() => {
            let preserved = store.root().join(format!(
                "gamesound.corrupt-{}.db",
                chrono::Local::now().format("%Y%m%d%H%M%S")
            ));
            std::fs::rename(store.db_path(), &preserved).with_context(|| {
                format!("database could not be opened and could not be preserved: {error}")
            })?;
            Library::open(&store.db_path()).with_context(|| {
                format!(
                    "database was reset after corruption (preserved at {}); restore a backup if needed",
                    preserved.display()
                )
            })
        }
        Err(error) => Err(error),
    }
}

/// Restores the user's terminal even if terminal creation, drawing, or input
/// fails while the application is active.
struct TerminalGuard;
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, Show);
    }
}
fn event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> Result<()> {
    while !app.quit {
        terminal.draw(|f| ui::draw(f, &app))?;
        if event::poll(Duration::from_millis(app.config.tui.tick_rate_ms))? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key);
            }
        }
        pump_platform_events();
        app.pump_runtime();
    }
    app.shutdown();
    Ok(())
}

/// Carbon hotkeys on macOS are delivered through the application CFRunLoop.
/// Crossterm owns terminal input, so explicitly service pending platform events
/// once per TUI tick without blocking terminal rendering.
#[cfg(target_os = "macos")]
fn pump_platform_events() {
    unsafe {
        core_foundation_sys::runloop::CFRunLoopRunInMode(
            core_foundation_sys::runloop::kCFRunLoopDefaultMode,
            0.0,
            1,
        );
    }
}
#[cfg(not(target_os = "macos"))]
fn pump_platform_events() {}
