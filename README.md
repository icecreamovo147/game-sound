# GameSound

GameSound is a Rust soundboard for game and voice-chat use. It captures a real microphone, mixes it with triggered sound effects, and outputs the result through a virtual audio output device — making your voice and sound effects audible together in Discord, QQ, WeChat, or any in-game voice chat.

## Editions

| Edition | Interface | Tech Stack | Entry Point |
|---------|-----------|------------|-------------|
| **game-sound** | Terminal (TUI) | Rust + Ratatui + Crossterm | `cargo run -p gamesound` |
| **game-sound-desktop** | Desktop (GUI) | Tauri 2 + React + Mantine | `cd crates/gamesound-desktop && pnpm tauri:dev` |

Both editions share the same core audio engine (`gamesound-core`) and persistent storage (`gamesound-storage`). Sounds, categories, profiles, hotkeys, and settings are stored once and available to either interface.

## Quick start — TUI

```bash
cargo run -p gamesound
# Or list devices and virtual-device hints
cargo run -p gamesound -- doctor
# Import a file or recursively scan a directory
cargo run -p gamesound -- import ./my-sounds
# Keep independent copies under the GameSound data directory
cargo run -p gamesound -- import --copy ./my-sounds
# Create a portable library manifest or a config/database snapshot
cargo run -p gamesound -- export gamesound-library.json
cargo run -p gamesound -- backup
```

On first launch open configuration (`C`), select a real microphone (`M`) and a virtual output (`O`), such as BlackHole on macOS or VB-CABLE on Windows. Select the corresponding virtual **input** in Discord, QQ, WeChat, or the game. Optional local monitoring uses a separate output (`L`). Start the device streams with `T`.

## Quick start — Desktop GUI

```bash
cd crates/gamesound-desktop

# Install frontend dependencies
pnpm install

# Run in development mode
pnpm tauri:dev

# Build for production
pnpm tauri:build
```

The desktop application provides:
- **Dashboard** — Real-time status overview, audio levels, quick actions
- **Sound Library** — Card-based sound browser with search, categories, play/stop, volume control
- **Device Settings** — Enumerate, select, and test audio devices with virtual device detection
- **Mixer** — Per-channel volume, mute, ducking, and real-time level meters
- **Hotkeys** — Bind global shortcuts to sounds with conflict detection and keyboard capture
- **Settings** — Theme, config export/import, log access, profile info
- **Setup Guide** — Step-by-step instructions for macOS, Windows, Discord, QQ, and in-game configuration

## TUI controls

- `A`: add a sound by entering a local path; `D`: remove its library record; `/`: search.
- `Enter`: play, `Space`: stop selected, `P`: pause/resume, `S`: stop all, `+/-`: adjust selected sound volume.
- `Tab`: move to Categories; there, `N` creates, `R` renames and `D` removes a category. Removing a category keeps its sounds in the library.
- The category sidebar also includes Favorites and Recently used; favorites and playback history are persisted per profile.
- `B`: persist and register a global sound hotkey, such as `ctrl+1`; `E`: rename a sound.
- In Settings, `P` switches an isolated sound profile and `N` creates one. Every profile owns its own sound list, categories and active global bindings.
- `M/O/L`: choose microphone, virtual output, and monitor. `C`: configuration, `?`: help, `Q`: quit.
- In Settings, `1/2/3` clears Mic Input, Virtual Output, or Monitor. `Z` switches and persists the interface language between English and Chinese.
- `J` opens the in-app runtime event log for stream, playback and hotkey errors.
- `:` opens a command panel: `:play <id>`, `:stop-all`, `:set <mic|output|monitor> <device>`, `:profile <name>`, and `:help`.

The library can either reference source paths (the default) or copy media into its managed sounds directory (`import --copy`). WAV, MP3, M4A, AAC, OGG and FLAC can be imported; directory imports recurse through subfolders. Invalid paths are visibly marked and rejected on playback/import. Configuration is stored under the platform-local GameSound application-data directory, with a corrupt TOML file preserved before defaults are regenerated.

`gamesound doctor` is read-only and reports enumerated devices, selected-device availability, likely virtual-output status, the current database migration version, and the active profile. Inside a restricted sandbox it may report no devices or an uninitialized data directory; that is diagnostic output, not a crash.

Exported profile manifests include categories, sound settings and per-sound global hotkeys. Importing a manifest remaps database IDs safely and leaves missing source media visible for repair rather than discarding its library entry.

## Global hotkeys and platform note

`gamesound-core` provides a system-wide `global-hotkey` registry, conflict detection, and parsers for combinations such as `ctrl+1` and `ctrl+alt+s`. macOS may require Accessibility permission. The TUI keeps its local keys separate from that registry. The output is deliberately a virtual **audio output**—software cannot write to a physical microphone.

## Manual audio acceptance

1. Install BlackHole (macOS) or VB-CABLE (Windows).
2. Run `gamesound doctor` and verify the device is listed (and preferably tagged virtual).
3. Select a real mic and virtual output in Settings, press `T`, then add a WAV/MP3 using `A` and play it.
4. Set the virtual device as the voice app’s input. Confirm speech and the effect are both audible.
5. Enable a monitor device only with headphones to avoid feedback. Disconnect a selected device to verify the TUI reports the stream error rather than crashing.

The automated test suite covers mixer limiting/ducking, format/path failures, configuration persistence/backup/recovery, category-safe deletion, recursive imports, virtual-device recognition, and hotkey parsing. Physical routing, driver availability, and OS hotkey permissions require the above manual check.

## Build & Test

```bash
# Lint and format check
cargo fmt --check
cargo clippy --workspace --all-targets --all-features

# Run all tests
cargo test --workspace --all-features

# Build all crates (TUI + Desktop + Core + Storage)
cargo build --workspace

# Desktop frontend
cd crates/gamesound-desktop
pnpm install
pnpm typecheck
pnpm build

# Full Tauri build (requires platform-specific setup)
pnpm tauri:build
```

## Architecture

```
gamesound/
├── crates/
│   ├── gamesound-core/        Audio engine, device I/O, hotkey registry, mixer, runtime
│   ├── gamesound-storage/     SQLite library, TOML config, backup/restore
│   ├── gamesound-tui/         Terminal UI (Ratatui + Crossterm)
│   └── gamesound-desktop/     Desktop GUI (Tauri 2 + React + Mantine)
│       ├── src/               React frontend (TypeScript)
│       │   ├── api/           Tauri invoke wrappers
│       │   ├── stores/        Zustand state management
│       │   ├── pages/         Dashboard, SoundLibrary, Devices, Mixer, Hotkeys, Settings, SetupGuide
│       │   └── components/    Layout, Navbar, LevelMeter, SoundCard, DeviceSelector, etc.
│       └── src-tauri/         Rust backend
│           └── src/
│               ├── commands/  All Tauri commands (runtime, sounds, devices, mixer, hotkeys, settings)
│               ├── state.rs   Shared AppState (RuntimeHandle, Library, ConfigStore)
│               └── events.rs  Runtime event forwarding to frontend
└── docs/                      Project documentation (13 docs, EN/ZH)
```

## Virtual Audio Device Setup

**macOS:** Install [BlackHole 2ch](https://existential.audio/blackhole/)  
**Windows:** Install [VB-CABLE](https://vb-audio.com/Cable/)

In GameSound, select your real microphone and the virtual device as output. Then set the virtual device as the input in your voice apps. See the **Setup Guide** page in the desktop app for detailed instructions.
