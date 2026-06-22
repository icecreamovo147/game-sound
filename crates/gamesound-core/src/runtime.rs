use crate::{
    audio::{AudioEngine, AudioEvent, PreparedSound, StreamPreferences},
    hotkey::HotkeyRegistry,
    mixer::{Levels, MixerSettings},
    sound::Sound,
};
#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

use crossbeam_channel::{unbounded, Receiver, Sender};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeStatus {
    Stopped,
    Running,
    Warning,
}
#[derive(Debug, Clone, Copy)]
pub enum VolumeTarget {
    Mic,
    Sfx,
    Monitor,
}
#[derive(Debug, Clone)]
pub enum RuntimeCommand {
    PlaySound(Sound),
    StopSound(i64),
    PauseSound(i64),
    ResumeSound(i64),
    StopAll,
    Start {
        mic: Option<String>,
        output: String,
        monitor: Option<String>,
        monitor_sfx_only: bool,
        preferences: StreamPreferences,
    },
    SetVolume {
        target: VolumeTarget,
        value: f32,
    },
    SetMute {
        target: VolumeTarget,
        muted: bool,
    },
    SetDucking {
        enabled: bool,
        ratio: f32,
        attack_ms: u32,
        release_ms: u32,
        release_delay_ms: u32,
    },
    SetHotkeys {
        sounds: Vec<(String, Sound)>,
        stop_all: String,
        toggle_mic: String,
        toggle_sfx: String,
        toggle_monitor: String,
        sfx_volume_up: String,
        sfx_volume_down: String,
        mic_volume_up: String,
        mic_volume_down: String,
        monitor_volume_up: String,
        monitor_volume_down: String,
        switch_profile: String,
    },
    SuspendHotkeys,
    StopAudio,
    Shutdown(std::sync::mpsc::Sender<()>),
}
#[derive(Debug, Clone)]
pub enum RuntimeEvent {
    SoundStarted(i64),
    SoundStopped(i64),
    HotkeysSuspended,
    HotkeysRegistered(usize),
    HotkeyCaptured(String),
    Levels(Levels),
    Status(RuntimeStatus),
    Error(String),
    Warning(String),
    SwitchProfileRequested,
}
#[derive(Clone)]
pub struct RuntimeHandle {
    pub commands: Sender<RuntimeCommand>,
    pub events: Receiver<RuntimeEvent>,
}
pub fn spawn_runtime() -> RuntimeHandle {
    tracing::info!(target: "gamesound_core::runtime", "spawning runtime thread");
    let (cmd_tx, cmd_rx) = unbounded();
    let (event_tx, event_rx) = unbounded();
    let (raw_hotkey_tx, raw_hotkey_rx) = unbounded();
    #[cfg(target_os = "macos")]
    start_macos_hotkey_listener(raw_hotkey_tx, event_tx.clone());
    thread::Builder::new()
        .name("gamesound-runtime".into())
        .spawn(move || run(cmd_rx, event_tx, raw_hotkey_rx))
        .expect("runtime thread");
    RuntimeHandle {
        commands: cmd_tx,
        events: event_rx,
    }
}
fn run(
    commands: Receiver<RuntimeCommand>,
    events: Sender<RuntimeEvent>,
    raw_hotkeys: Receiver<String>,
) {
    // CPAL streams are intentionally !Send on some hosts (notably CoreAudio), so
    // the runtime thread owns the engine and emits level snapshots after commands.
    tracing::info!(target: "gamesound_core::runtime", "runtime loop started");
    let mut engine = AudioEngine::default();
    let mut settings = MixerSettings::default();
    let mut hotkeys: Option<HotkeyRegistry> = None;
    let mut hotkey_actions: HashMap<u32, HotkeyAction> = HashMap::new();
    let mut hotkey_shortcuts: HashMap<String, HotkeyAction> = HashMap::new();
    let mut hotkey_capture = false;
    let (prepared_tx, prepared_rx) = unbounded::<(u64, i64, anyhow::Result<PreparedSound>)>();
    let mut next_request_id = 1u64;
    let mut pending_requests = HashMap::<u64, i64>::new();
    let mut cancelled_requests = HashSet::<u64>::new();
    loop {
        emit_audio_events(&mut engine, &events);
        while let Ok((request_id, _id, prepared)) = prepared_rx.try_recv() {
            pending_requests.remove(&request_id);
            if cancelled_requests.remove(&request_id) {
                continue;
            }
            match prepared {
                Ok(prepared) => {
                    if let Err(error) = engine.play_prepared(prepared) {
                        let _ = events.send(RuntimeEvent::Error(error.to_string()));
                    }
                }
                Err(error) => {
                    let _ = events.send(RuntimeEvent::Error(error.to_string()));
                }
            }
        }
        #[cfg(target_os = "macos")]
        while let Ok(shortcut) = raw_hotkeys.try_recv() {
            if hotkey_capture {
                hotkey_capture = false;
                let _ = events.send(RuntimeEvent::HotkeyCaptured(shortcut));
            } else if let Some(action) = hotkey_shortcuts.get(&shortcut).cloned() {
                dispatch_hotkey(
                    action,
                    &mut engine,
                    &mut settings,
                    &events,
                    &prepared_tx,
                    &mut next_request_id,
                    &mut pending_requests,
                );
            }
        }
        if let Some(registry) = hotkeys.as_ref() {
            while let Ok(event) = global_hotkey::GlobalHotKeyEvent::receiver().try_recv() {
                if matches!(event.state, global_hotkey::HotKeyState::Pressed) {
                    if let Some(action) = hotkey_actions.get(&event.id).cloned() {
                        dispatch_hotkey(
                            action,
                            &mut engine,
                            &mut settings,
                            &events,
                            &prepared_tx,
                            &mut next_request_id,
                            &mut pending_requests,
                        );
                    }
                }
            }
            let _ = registry;
        }
        let command = crossbeam_channel::select! {
            recv(prepared_rx) -> message => match message {
                Ok((request_id, _id, prepared)) => {
                    pending_requests.remove(&request_id);
                    if !cancelled_requests.remove(&request_id) {
                        match prepared {
                            Ok(prepared) => { if let Err(error) = engine.play_prepared(prepared) { let _ = events.send(RuntimeEvent::Error(error.to_string())); } }
                            Err(error) => { let _ = events.send(RuntimeEvent::Error(error.to_string())); }
                        }
                    }
                    continue;
                }
                Err(_) => break,
            },
            recv(commands) -> command => match command {
                Ok(command) => command,
                Err(_) => break,
            },
            default(Duration::from_millis(100)) => {
                emit_audio_events(&mut engine, &events);
                let _ = events.send(RuntimeEvent::Levels(engine.levels()));
                continue;
            },
        };
        let result = match command {
            RuntimeCommand::PlaySound(s) => {
                let (rate, channels) = engine.output_format();
                let sender = prepared_tx.clone();
                let id = s.id;
                let request_id = next_request_id;
                next_request_id += 1;
                pending_requests.insert(request_id, id);
                thread::Builder::new()
                    .name("gamesound-decode".into())
                    .spawn(move || {
                        let _ =
                            sender.send((request_id, id, AudioEngine::prepare(s, rate, channels)));
                    })
                    .map(|_| ())
                    .map_err(Into::into)
            }
            RuntimeCommand::StopSound(id) => {
                cancel_pending_for_sound(&pending_requests, &mut cancelled_requests, id);
                engine.stop_sound(id)
            }
            RuntimeCommand::PauseSound(id) => engine.set_paused(id, true),
            RuntimeCommand::ResumeSound(id) => engine.set_paused(id, false),
            RuntimeCommand::StopAll => {
                cancelled_requests.extend(pending_requests.keys().copied());
                engine.stop_all()
            }
            RuntimeCommand::Start {
                mic,
                output,
                monitor,
                monitor_sfx_only,
                preferences,
            } => engine
                .start(
                    mic.as_deref(),
                    &output,
                    monitor.as_deref(),
                    monitor_sfx_only,
                    preferences,
                )
                .map(|_| {
                    let _ = events.send(RuntimeEvent::Status(RuntimeStatus::Running));
                }),
            RuntimeCommand::SetVolume { target, value } => {
                match target {
                    VolumeTarget::Mic => settings.mic_volume = value.clamp(0., 1.),
                    VolumeTarget::Sfx => settings.sfx_volume = value.clamp(0., 1.),
                    VolumeTarget::Monitor => settings.monitor_volume = value.clamp(0., 1.),
                };
                engine.set_settings(settings);
                Ok(())
            }
            RuntimeCommand::SetMute { target, muted } => {
                match target {
                    VolumeTarget::Mic => settings.mic_muted = muted,
                    VolumeTarget::Sfx => settings.sfx_muted = muted,
                    VolumeTarget::Monitor => settings.monitor_muted = muted,
                };
                engine.set_settings(settings);
                Ok(())
            }
            RuntimeCommand::SetDucking {
                enabled,
                ratio,
                attack_ms,
                release_ms,
                release_delay_ms,
            } => {
                settings.ducking = enabled;
                settings.duck_ratio = ratio.clamp(0.0, 1.0);
                settings.duck_attack_ms = attack_ms;
                settings.duck_release_ms = release_ms;
                settings.duck_release_delay_ms = release_delay_ms;
                engine.set_settings(settings);
                Ok(())
            }
            RuntimeCommand::SetHotkeys {
                sounds,
                stop_all,
                toggle_mic,
                toggle_sfx,
                toggle_monitor,
                sfx_volume_up,
                sfx_volume_down,
                mic_volume_up,
                mic_volume_down,
                monitor_volume_up,
                monitor_volume_down,
                switch_profile,
            } => (|| -> anyhow::Result<()> {
                let old_bindings = hotkeys
                    .as_ref()
                    .map(|registry| {
                        hotkey_actions
                            .iter()
                            .filter_map(|(id, action)| {
                                registry
                                    .action_for(*id)
                                    .map(|shortcut| (shortcut.to_owned(), action.clone()))
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                // Parse and de-duplicate all requested shortcuts before any active
                // system registration is touched.
                let mut requested = sounds
                    .iter()
                    .map(|(shortcut, _)| shortcut.to_ascii_lowercase())
                    .collect::<Vec<_>>();
                requested.extend([
                    stop_all.to_ascii_lowercase(),
                    toggle_mic.to_ascii_lowercase(),
                    toggle_sfx.to_ascii_lowercase(),
                    toggle_monitor.to_ascii_lowercase(),
                    sfx_volume_up.to_ascii_lowercase(),
                    sfx_volume_down.to_ascii_lowercase(),
                    mic_volume_up.to_ascii_lowercase(),
                    mic_volume_down.to_ascii_lowercase(),
                    monitor_volume_up.to_ascii_lowercase(),
                    monitor_volume_down.to_ascii_lowercase(),
                    switch_profile.to_ascii_lowercase(),
                ]);
                for shortcut in &requested {
                    crate::hotkey::parse(shortcut)?;
                }
                if requested
                    .iter()
                    .collect::<std::collections::HashSet<_>>()
                    .len()
                    != requested.len()
                {
                    anyhow::bail!(
                        "hotkey conflict: a shortcut is assigned to more than one action"
                    );
                }
                if let Some(mut previous) = hotkeys.take() {
                    previous.unregister_all()?;
                }
                let mut registry = HotkeyRegistry::new()?;
                let mut registered = HashMap::new();
                let mut shortcuts = HashMap::new();
                let register_result = (|| -> anyhow::Result<()> {
                    for (shortcut, sound) in sounds {
                        let id = registry.register(&shortcut)?;
                        let action = HotkeyAction::Play(sound);
                        registered.insert(id, action.clone());
                        shortcuts.insert(shortcut.to_ascii_lowercase(), action);
                    }
                    for (shortcut, action) in [
                        (stop_all, HotkeyAction::StopAll),
                        (toggle_mic, HotkeyAction::ToggleMic),
                        (toggle_sfx, HotkeyAction::ToggleSfx),
                        (toggle_monitor, HotkeyAction::ToggleMonitor),
                        (sfx_volume_up, HotkeyAction::Adjust(VolumeTarget::Sfx, 0.05)),
                        (
                            sfx_volume_down,
                            HotkeyAction::Adjust(VolumeTarget::Sfx, -0.05),
                        ),
                        (mic_volume_up, HotkeyAction::Adjust(VolumeTarget::Mic, 0.05)),
                        (
                            mic_volume_down,
                            HotkeyAction::Adjust(VolumeTarget::Mic, -0.05),
                        ),
                        (
                            monitor_volume_up,
                            HotkeyAction::Adjust(VolumeTarget::Monitor, 0.05),
                        ),
                        (
                            monitor_volume_down,
                            HotkeyAction::Adjust(VolumeTarget::Monitor, -0.05),
                        ),
                        (switch_profile, HotkeyAction::SwitchProfile),
                    ] {
                        let id = registry.register(&shortcut)?;
                        registered.insert(id, action.clone());
                        shortcuts.insert(shortcut.to_ascii_lowercase(), action);
                    }
                    Ok(())
                })();
                if let Err(error) = register_result {
                    let _ = registry.unregister_all();
                    let mut restored = HotkeyRegistry::new()?;
                    let mut restored_actions = HashMap::new();
                    for (shortcut, action) in old_bindings {
                        let id = restored.register(&shortcut)?;
                        restored_actions.insert(id, action);
                    }
                    hotkeys = Some(restored);
                    hotkey_actions = restored_actions;
                    return Err(error);
                }
                hotkeys = Some(registry);
                hotkey_actions = registered;
                hotkey_shortcuts = shortcuts;
                hotkey_capture = false;
                let _ = events.send(RuntimeEvent::HotkeysRegistered(hotkey_actions.len()));
                Ok(())
            })(),
            RuntimeCommand::SuspendHotkeys => (|| -> anyhow::Result<()> {
                if let Some(mut registry) = hotkeys.take() {
                    registry.unregister_all()?;
                }
                hotkey_actions.clear();
                hotkey_shortcuts.clear();
                hotkey_capture = true;
                let _ = events.send(RuntimeEvent::HotkeysSuspended);
                Ok(())
            })(),
            RuntimeCommand::StopAudio => {
                engine.shutdown();
                let _ = events.send(RuntimeEvent::Status(RuntimeStatus::Stopped));
                Ok(())
            }
            RuntimeCommand::Shutdown(done) => {
                tracing::info!(target: "gamesound_core::runtime", "shutdown requested, stopping engine");
                engine.shutdown();
                let _ = events.send(RuntimeEvent::Status(RuntimeStatus::Stopped));
                let _ = done.send(());
                break;
            }
        };
        if let Err(e) = result {
            let _ = events.send(RuntimeEvent::Error(e.to_string()));
        }
        emit_audio_events(&mut engine, &events);
        let _ = events.send(RuntimeEvent::Levels(engine.levels()));
    }
}
#[derive(Clone)]
enum HotkeyAction {
    Play(Sound),
    StopAll,
    ToggleMic,
    ToggleSfx,
    ToggleMonitor,
    Adjust(VolumeTarget, f32),
    SwitchProfile,
}
fn dispatch_hotkey(
    action: HotkeyAction,
    engine: &mut AudioEngine,
    settings: &mut MixerSettings,
    _events: &Sender<RuntimeEvent>,
    prepared_tx: &Sender<(u64, i64, anyhow::Result<PreparedSound>)>,
    next_request_id: &mut u64,
    pending_requests: &mut HashMap<u64, i64>,
) {
    match action {
        HotkeyAction::Play(sound) => {
            let (rate, channels) = engine.output_format();
            let sender = prepared_tx.clone();
            let id = sound.id;
            let request_id = *next_request_id;
            *next_request_id += 1;
            pending_requests.insert(request_id, id);
            thread::spawn(move || {
                let _ = sender.send((request_id, id, AudioEngine::prepare(sound, rate, channels)));
            });
        }
        HotkeyAction::StopAll => {
            let _ = engine.stop_all();
        }
        HotkeyAction::ToggleMic => {
            settings.mic_muted = !settings.mic_muted;
            engine.set_settings(*settings);
        }
        HotkeyAction::ToggleSfx => {
            settings.sfx_muted = !settings.sfx_muted;
            engine.set_settings(*settings);
        }
        HotkeyAction::ToggleMonitor => {
            settings.monitor_muted = !settings.monitor_muted;
            engine.set_settings(*settings);
        }
        HotkeyAction::Adjust(target, delta) => {
            match target {
                VolumeTarget::Mic => {
                    settings.mic_volume = (settings.mic_volume + delta).clamp(0.0, 1.0)
                }
                VolumeTarget::Sfx => {
                    settings.sfx_volume = (settings.sfx_volume + delta).clamp(0.0, 1.0)
                }
                VolumeTarget::Monitor => {
                    settings.monitor_volume = (settings.monitor_volume + delta).clamp(0.0, 1.0)
                }
            }
            engine.set_settings(*settings);
        }
        HotkeyAction::SwitchProfile => {
            let _ = _events.send(RuntimeEvent::SwitchProfileRequested);
        }
    }
}

fn emit_audio_events(engine: &mut AudioEngine, events: &Sender<RuntimeEvent>) {
    for event in engine.drain_events() {
        match event {
            AudioEvent::SoundStarted(id) => {
                let _ = events.send(RuntimeEvent::SoundStarted(id));
            }
            AudioEvent::SoundStopped(id) => {
                let _ = events.send(RuntimeEvent::SoundStopped(id));
            }
            AudioEvent::MicOverrun => {
                let _ = events.send(RuntimeEvent::Warning(
                    "microphone buffer overrun; audio was dropped to protect real-time output"
                        .into(),
                ));
            }
            AudioEvent::MonitorOverrun => {
                let _ = events.send(RuntimeEvent::Warning(
                    "monitor buffer overrun; local monitoring dropped frames".into(),
                ));
            }
            AudioEvent::MicUnderflow(count) => {
                tracing::trace!(
                    target: "gamesound_core::runtime",
                    count,
                    "mic ring buffer underflow ({} frames dropped this callback cycle)", count
                );
            }
        }
    }
    let health = engine.take_health();
    if health.mic_fault {
        let _ = events.send(RuntimeEvent::Error(
            "microphone stream failed or was disconnected; choose the microphone again".into(),
        ));
    }
    if health.output_fault {
        let _ = events.send(RuntimeEvent::Error(
            "virtual output stream failed or was disconnected; choose the output device again"
                .into(),
        ));
    }
    if health.monitor_fault {
        let _ = events.send(RuntimeEvent::Warning(
            "monitor stream failed or was disconnected; choose the monitor device again".into(),
        ));
    }
}

fn cancel_pending_for_sound(
    pending_requests: &HashMap<u64, i64>,
    cancelled_requests: &mut HashSet<u64>,
    sound_id: i64,
) {
    cancelled_requests.extend(pending_requests.iter().filter_map(
        |(request_id, pending_sound_id)| (*pending_sound_id == sound_id).then_some(*request_id),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancelling_one_sound_preserves_other_pending_decodes() {
        let pending = HashMap::from([(1, 10), (2, 20), (3, 10)]);
        let mut cancelled = HashSet::new();
        cancel_pending_for_sound(&pending, &mut cancelled, 10);
        assert_eq!(cancelled, HashSet::from([1, 3]));
    }
}

#[cfg(target_os = "macos")]
pub(crate) extern "C" fn hotkey_sigtrap_handler(_: libc::c_int) {
    // Terminate only the hotkey-listener thread; the main process survives.
    // This is called when macOS sends SIGTRAP because CGEventTap detected
    // that the process does not actually have Accessibility permissions
    // (common when AXIsProcessTrusted() returns a stale true).
    unsafe { libc::pthread_exit(std::ptr::null_mut()) };
}

#[cfg(target_os = "macos")]
pub(crate) fn install_hotkey_sigtrap_handler() {
    unsafe {
        let mut sa: libc::sigaction = std::mem::zeroed();
        sa.sa_sigaction = hotkey_sigtrap_handler as *const () as usize;
        sa.sa_flags = libc::SA_SIGINFO;
        libc::sigaction(libc::SIGTRAP, &sa, std::ptr::null_mut());
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn restore_sigtrap_handler() {
    unsafe {
        libc::signal(libc::SIGTRAP, libc::SIG_DFL);
    }
}

#[cfg(target_os = "macos")]
fn start_macos_hotkey_listener(sender: Sender<String>, events: Sender<RuntimeEvent>) {
    thread::Builder::new()
        .name("gamesound-macos-hotkeys".into())
        .spawn(move || {
            let trusted = unsafe { AXIsProcessTrusted() };
            if !trusted {
                let _ = events.send(RuntimeEvent::Warning(
                    "macOS Accessibility permission not granted. Global hotkeys and key capture are disabled.\n\n                     To enable: open System Settings \u{2192} Privacy & Security \u{2192} Accessibility,                      then add and enable GameSound Desktop.\n\n                     If the app is not listed, click the '+' button and navigate to the app bundle,                      or drag the app icon from Finder into the list.".into(),
                ));
                return;
            }

            // Install a SIGTRAP handler that cleanly terminates only this thread.
            // rdev::listen uses CGEventTap which may send SIGTRAP if permissions
            // were granted to a previous build but are now invalid (common in dev).
            install_hotkey_sigtrap_handler();

            // Use catch_unwind so that rdev panics do not crash the entire process.
            let result = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                let _ = events.send(RuntimeEvent::Warning("macOS global hotkeys use the Accessibility keyboard listener; grant Accessibility permission to the host app (VS Code or Terminal) if no events arrive".into()));
                let modifiers = Arc::new(Mutex::new(RawModifiers::default()));
                let state = modifiers.clone();
                if let Err(error) = rdev::listen(move |event| match event.event_type {
                    rdev::EventType::KeyPress(key) => {
                        let mut modifiers = state.lock().expect("modifier mutex");
                        if modifiers.press(key) {
                            return;
                        }
                        if let Some(key_name) = rdev_key_name(key) {
                            let _ = sender.send(modifiers.format(key_name));
                        }
                    }
                    rdev::EventType::KeyRelease(key) => {
                        state.lock().expect("modifier mutex").release(key);
                    }
                    _ => {}
                }) {
                    let _ = events.send(RuntimeEvent::Error(format!(
                        "macOS global keyboard listener failed: {error:?}"
                    )));
                }
            }));

            // Restore default SIGTRAP handler for other parts of the process
            restore_sigtrap_handler();

            if let Err(panic) = result {
                let msg = panic.downcast_ref::<String>().map(String::as_str)
                    .or_else(|| panic.downcast_ref::<&str>().copied())
                    .unwrap_or("unknown panic");
                let _ = events.send(RuntimeEvent::Error(format!(
                    "macOS global keyboard listener panicked: {msg}"
                )));
            }
        })
        .expect("macOS hotkey thread");
}
#[cfg(target_os = "macos")]
#[derive(Default)]
struct RawModifiers {
    ctrl: bool,
    alt: bool,
    shift: bool,
    meta: bool,
}
#[cfg(target_os = "macos")]
impl RawModifiers {
    fn press(&mut self, key: rdev::Key) -> bool {
        match key {
            rdev::Key::ControlLeft | rdev::Key::ControlRight => self.ctrl = true,
            rdev::Key::Alt | rdev::Key::AltGr => self.alt = true,
            rdev::Key::ShiftLeft | rdev::Key::ShiftRight => self.shift = true,
            rdev::Key::MetaLeft | rdev::Key::MetaRight => self.meta = true,
            _ => return false,
        };
        true
    }
    fn release(&mut self, key: rdev::Key) {
        match key {
            rdev::Key::ControlLeft | rdev::Key::ControlRight => self.ctrl = false,
            rdev::Key::Alt | rdev::Key::AltGr => self.alt = false,
            rdev::Key::ShiftLeft | rdev::Key::ShiftRight => self.shift = false,
            rdev::Key::MetaLeft | rdev::Key::MetaRight => self.meta = false,
            _ => {}
        }
    }
    fn format(&self, key: &str) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("ctrl")
        };
        if self.alt {
            parts.push("alt")
        };
        if self.shift {
            parts.push("shift")
        };
        if self.meta {
            parts.push("meta")
        };
        parts.push(key);
        parts.join("+")
    }
}
#[cfg(target_os = "macos")]
fn rdev_key_name(key: rdev::Key) -> Option<&'static str> {
    use rdev::Key::*;
    Some(match key {
        KeyA => "a",
        KeyB => "b",
        KeyC => "c",
        KeyD => "d",
        KeyE => "e",
        KeyF => "f",
        KeyG => "g",
        KeyH => "h",
        KeyI => "i",
        KeyJ => "j",
        KeyK => "k",
        KeyL => "l",
        KeyM => "m",
        KeyN => "n",
        KeyO => "o",
        KeyP => "p",
        KeyQ => "q",
        KeyR => "r",
        KeyS => "s",
        KeyT => "t",
        KeyU => "u",
        KeyV => "v",
        KeyW => "w",
        KeyX => "x",
        KeyY => "y",
        KeyZ => "z",
        Num0 => "0",
        Num1 => "1",
        Num2 => "2",
        Num3 => "3",
        Num4 => "4",
        Num5 => "5",
        Num6 => "6",
        Num7 => "7",
        Num8 => "8",
        Num9 => "9",
        F1 => "f1",
        F2 => "f2",
        F3 => "f3",
        F4 => "f4",
        F5 => "f5",
        F6 => "f6",
        F7 => "f7",
        F8 => "f8",
        F9 => "f9",
        F10 => "f10",
        F11 => "f11",
        F12 => "f12",
        Space => "space",
        UpArrow => "up",
        DownArrow => "down",
        LeftArrow => "left",
        RightArrow => "right",
        _ => return None,
    })
}
