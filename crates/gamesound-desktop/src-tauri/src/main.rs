// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Logging is initialised inside run() once the config directory is known.
    // Early-boot failures are printed to stderr via eprintln!.
    gamesound_desktop::run()
}
