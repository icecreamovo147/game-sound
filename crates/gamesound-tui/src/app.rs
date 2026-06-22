#[cfg(any(not(target_os = "macos"), test))]
use crossterm::event::KeyModifiers;
use crossterm::event::{KeyCode, KeyEvent};
use gamesound_core::{
    audio::StreamPreferences,
    device,
    runtime::{spawn_runtime, RuntimeCommand, RuntimeEvent, RuntimeStatus, VolumeTarget},
    Sound,
};
use gamesound_storage::{AppConfig, ConfigStore, Language, Library, MonitorMode, Profile};
use std::collections::{HashSet, VecDeque};
#[derive(Clone, Copy, PartialEq)]
pub enum Page {
    Main,
    Onboarding,
    Help,
    Config,
    Device(DeviceKind),
    ProfilePicker,
    Logs,
}
#[derive(Clone, Copy, PartialEq)]
pub enum Focus {
    Categories,
    Sounds,
}
#[derive(Clone, Copy, PartialEq)]
pub enum SpecialFilter {
    Favorites,
    Recent,
}
/// Converts Crossterm's observed key event into the same portable syntax the
/// global-hotkey layer persists and registers (for example `ctrl+shift+k`).
#[cfg(any(not(target_os = "macos"), test))]
fn format_hotkey(key: KeyEvent) -> Option<String> {
    let key_name = match key.code {
        KeyCode::Char(character) if character.is_ascii_alphanumeric() => {
            character.to_ascii_lowercase().to_string()
        }
        KeyCode::Up => "up".into(),
        KeyCode::Down => "down".into(),
        KeyCode::Left => "left".into(),
        KeyCode::Right => "right".into(),
        KeyCode::Char(' ') => "space".into(),
        KeyCode::F(number @ 1..=12) => format!("f{number}"),
        _ => return None,
    };
    let mut parts = Vec::new();
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("ctrl");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        parts.push("alt");
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        parts.push("shift");
    }
    if key.modifiers.contains(KeyModifiers::SUPER) {
        parts.push("meta");
    }
    parts.push(&key_name);
    Some(parts.join("+"))
}
#[derive(Clone, Copy, PartialEq)]
pub enum DeviceKind {
    Mic,
    Output,
    Monitor,
}
pub struct App {
    pub library: Library,
    pub store: ConfigStore,
    pub config: AppConfig,
    pub sounds: Vec<Sound>,
    pub categories: Vec<gamesound_core::sound::Category>,
    pub profiles: Vec<Profile>,
    pub active_profile_id: i64,
    pub selected: usize,
    pub selected_category: usize,
    pub selected_device: usize,
    pub selected_profile: usize,
    pub category: Option<i64>,
    pub special_filter: Option<SpecialFilter>,
    pub focus: Focus,
    pub page: Page,
    pub query: String,
    pub input: Option<String>,
    pub capture_hotkey: Option<i64>,
    pub pending_hotkey_capture: Option<i64>,
    pub notice: String,
    pub quit: bool,
    pub runtime: gamesound_core::RuntimeHandle,
    pub playing: Option<i64>,
    pub active_sounds: HashSet<i64>,
    pub runtime_status: RuntimeStatus,
    pub pending_delete: Option<i64>,
    pub paused: bool,
    pub levels: gamesound_core::mixer::Levels,
    pub devices: Vec<gamesound_core::device::AudioDevice>,
    pub logs: VecDeque<String>,
}
impl App {
    pub fn is_chinese(&self) -> bool {
        self.config.tui.language == Language::Chinese
    }
    pub fn new(library: Library, store: ConfigStore, config: AppConfig) -> Self {
        let runtime = spawn_runtime();
        let first_run = config.app.first_run;
        let active_profile = library
            .active_profile()
            .expect("default profile is created with the database");
        let cleared_reserved_hotkeys = library.clear_reserved_tui_hotkeys().unwrap_or(0);
        let profiles = library.profiles().unwrap_or_default();
        let selected_profile = profiles
            .iter()
            .position(|profile| profile.id == active_profile.id)
            .unwrap_or(0);
        let mut a = Self {
            library,
            store,
            config,
            sounds: vec![],
            categories: vec![],
            profiles,
            active_profile_id: active_profile.id,
            selected: 0,
            selected_category: 0,
            selected_device: 0,
            selected_profile,
            category: None,
            special_filter: None,
            focus: Focus::Sounds,
            page: if first_run {
                Page::Onboarding
            } else {
                Page::Main
            },
            query: String::new(),
            input: None,
            capture_hotkey: None,
            pending_hotkey_capture: None,
            notice: if cleared_reserved_hotkeys > 0 {
                format!("Removed {cleared_reserved_hotkeys} obsolete TUI-key hotkey binding(s)")
            } else {
                String::new()
            },
            quit: false,
            runtime,
            playing: None,
            active_sounds: HashSet::new(),
            runtime_status: RuntimeStatus::Stopped,
            pending_delete: None,
            paused: false,
            levels: Default::default(),
            devices: vec![],
            logs: VecDeque::new(),
        };
        a.reload();
        a.configure_hotkeys();
        a
    }
    pub fn reload(&mut self) {
        self.categories = self
            .library
            .categories_in_profile(self.active_profile_id)
            .unwrap_or_default();
        self.sounds = match self.special_filter {
            Some(SpecialFilter::Favorites) => self
                .library
                .favorite_sounds_in_profile(self.active_profile_id, &self.query),
            Some(SpecialFilter::Recent) => self
                .library
                .recent_sounds_in_profile(self.active_profile_id, &self.query),
            None => {
                self.library
                    .sounds_in_profile(self.active_profile_id, self.category, &self.query)
            }
        }
        .unwrap_or_default();
        self.selected = self.selected.min(self.sounds.len().saturating_sub(1));
        self.selected_category = self.selected_category.min(self.categories.len() + 2);
    }
    pub fn pump_runtime(&mut self) {
        while let Ok(event) = self.runtime.events.try_recv() {
            match event {
                RuntimeEvent::Levels(l) => self.levels = l,
                RuntimeEvent::Error(e) => {
                    self.runtime_status = RuntimeStatus::Warning;
                    self.push_log(format!("ERROR {e}"));
                    self.notice = format!("Error: {e}");
                }
                RuntimeEvent::Warning(w) => {
                    self.push_log(format!("WARN {w}"));
                    self.notice = format!("Warning: {w}");
                }
                RuntimeEvent::SoundStarted(id) => {
                    self.playing = Some(id);
                    self.active_sounds.insert(id);
                    self.push_log(format!("INFO started sound {id}"));
                }
                RuntimeEvent::SoundStopped(id) => {
                    if self.playing == Some(id) {
                        self.playing = None
                    }
                    self.active_sounds.remove(&id);
                    self.push_log(format!("INFO stopped sound {id}"));
                }
                RuntimeEvent::HotkeysSuspended => {
                    self.capture_hotkey = self.pending_hotkey_capture.take();
                    if self.capture_hotkey.is_some() {
                        self.notice = if self.is_chinese() {
                            "现在请按下新的快捷键组合（Esc 取消）"
                        } else {
                            "Now press a hotkey combination (Esc cancels)"
                        }
                        .into();
                    }
                }
                RuntimeEvent::HotkeysRegistered(count) => {
                    self.push_log(format!("INFO registered {count} global hotkeys"));
                    self.notice = if self.is_chinese() {
                        format!("已注册 {count} 个全局快捷键")
                    } else {
                        format!("Registered {count} global hotkeys")
                    };
                }
                RuntimeEvent::HotkeyCaptured(hotkey) => {
                    if let Some(sound_id) = self.capture_hotkey.take() {
                        self.apply_captured_hotkey(sound_id, hotkey);
                    }
                }
                RuntimeEvent::SwitchProfileRequested => self.switch_to_next_profile(),
                RuntimeEvent::Status(status) => {
                    self.runtime_status = status;
                    self.push_log(format!("INFO runtime {status:?}"));
                }
            }
        }
    }
    pub fn shutdown(&self) {
        let _ = self
            .runtime
            .commands
            .send(RuntimeCommand::Shutdown(std::sync::mpsc::channel().0));
    }
    fn selected_sound(&self) -> Option<Sound> {
        self.sounds.get(self.selected).cloned()
    }
    fn play(&mut self) {
        if let Some(s) = self.selected_sound() {
            let _ = self.runtime.commands.send(RuntimeCommand::PlaySound(s));
            if let Err(error) = self
                .library
                .record_play(self.sounds[self.selected].id, "tui")
            {
                self.notice = format!("Could not record playback: {error}");
            }
        } else {
            self.notice = "No sound selected".into();
        }
    }
    fn save(&mut self) {
        match self.store.save(&self.config) {
            Ok(_) => self.notice = "Configuration saved".into(),
            Err(e) => self.notice = format!("Cannot save configuration: {e}"),
        }
    }
    fn push_log(&mut self, line: String) {
        if self.logs.len() == 200 {
            self.logs.pop_front();
        }
        self.logs.push_back(format!(
            "{} {line}",
            chrono::Local::now().format("%H:%M:%S")
        ));
    }
    pub fn on_key(&mut self, key: KeyEvent) {
        if self.pending_hotkey_capture.is_some() {
            if key.code == KeyCode::Esc {
                self.pending_hotkey_capture = None;
                self.configure_hotkeys();
                self.notice = if self.is_chinese() {
                    "已取消快捷键绑定"
                } else {
                    "Hotkey binding cancelled"
                }
                .into();
            }
            return;
        }
        if let Some(sound_id) = self.capture_hotkey {
            self.capture_hotkey_key(sound_id, key);
            return;
        }
        if let Some(mut text) = self.input.take() {
            match key.code {
                KeyCode::Esc => self.notice = "Cancelled".into(),
                KeyCode::Enter => self.finish_input(text),
                KeyCode::Backspace => {
                    text.pop();
                    self.input = Some(text)
                }
                KeyCode::Char(c) => {
                    text.push(c);
                    self.input = Some(text)
                }
                _ => self.input = Some(text),
            };
            return;
        }
        match self.page {
            Page::Onboarding => self.onboarding_key(key),
            Page::Help => {
                if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
                    self.page = Page::Main
                }
            }
            Page::Config => self.config_key(key),
            Page::Device(kind) => self.device_key(kind, key),
            Page::ProfilePicker => self.profile_key(key),
            Page::Logs => {
                if matches!(
                    key.code,
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('j')
                ) {
                    self.page = Page::Main;
                }
            }
            Page::Main => self.main_key(key),
        }
    }
    fn onboarding_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('m') => self.open_devices(DeviceKind::Mic),
            KeyCode::Char('o') => self.open_devices(DeviceKind::Output),
            KeyCode::Char('l') => self.open_devices(DeviceKind::Monitor),
            KeyCode::Enter | KeyCode::Char('c') => self.page = Page::Config,
            KeyCode::Char('s') => {
                self.config.app.first_run = false;
                self.save();
                self.page = Page::Main;
            }
            _ => {}
        }
    }
    fn main_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.quit = true,
            KeyCode::Char('?') => self.page = Page::Help,
            KeyCode::Char('c') => self.page = Page::Config,
            KeyCode::Char('j') => self.page = Page::Logs,
            KeyCode::Tab => {
                self.focus = if self.focus == Focus::Sounds {
                    Focus::Categories
                } else {
                    Focus::Sounds
                }
            }
            KeyCode::Down if self.focus == Focus::Sounds => {
                self.selected = (self.selected + 1).min(self.sounds.len().saturating_sub(1));
            }
            KeyCode::Up if self.focus == Focus::Sounds => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Down if self.focus == Focus::Categories => {
                self.selected_category =
                    (self.selected_category + 1).min(self.categories.len() + 2);
            }
            KeyCode::Up if self.focus == Focus::Categories => {
                self.selected_category = self.selected_category.saturating_sub(1);
            }
            KeyCode::Enter if self.focus == Focus::Categories => {
                match self.selected_category {
                    0 => {
                        self.category = None;
                        self.special_filter = None;
                    }
                    1 => {
                        self.category = None;
                        self.special_filter = Some(SpecialFilter::Favorites);
                    }
                    2 => {
                        self.category = None;
                        self.special_filter = Some(SpecialFilter::Recent);
                    }
                    index => {
                        self.category = self.categories.get(index - 3).map(|category| category.id);
                        self.special_filter = None;
                    }
                }
                self.reload();
            }
            KeyCode::Enter => self.play(),
            KeyCode::Char(' ') => {
                if let Some(s) = self.selected_sound() {
                    let _ = self.runtime.commands.send(RuntimeCommand::StopSound(s.id));
                }
            }
            KeyCode::Char('p') => {
                if let Some(sound) = self.selected_sound() {
                    let command = if self.paused {
                        RuntimeCommand::ResumeSound(sound.id)
                    } else {
                        RuntimeCommand::PauseSound(sound.id)
                    };
                    let _ = self.runtime.commands.send(command);
                    self.paused = !self.paused;
                    self.notice = if self.paused {
                        "Sound paused"
                    } else {
                        "Sound resumed"
                    }
                    .into();
                }
            }
            KeyCode::Char('s') => {
                let _ = self.runtime.commands.send(RuntimeCommand::StopAll);
                self.playing = None
            }
            KeyCode::Char('/') => {
                self.input = Some(self.query.clone());
                self.notice = "Search: type text then Enter".into()
            }
            KeyCode::Char(':') => {
                self.input = Some(String::from(":"));
                self.notice = "Command: :play <id> | :stop-all | :set <mic|output|monitor> <device> | :profile <name>".into();
            }
            KeyCode::Char('a') => {
                self.input = Some(String::new());
                self.notice = "Add sound: enter an audio file path".into()
            }
            KeyCode::Char('d') if self.focus == Focus::Categories && self.selected_category > 2 => {
                if let Some(category) = self.categories.get(self.selected_category - 3) {
                    match self.library.remove_category(category.id) {
                        Ok(()) => {
                            self.category = None;
                            self.special_filter = None;
                            self.selected_category = 0;
                            self.notice =
                                "Category removed; its sounds are now uncategorized".into();
                            self.reload();
                        }
                        Err(error) => self.notice = format!("Category removal failed: {error}"),
                    }
                }
            }
            KeyCode::Char('d') => {
                if let Some(s) = self.selected_sound() {
                    if self.config.tui.confirm_on_delete && self.pending_delete != Some(s.id) {
                        self.pending_delete = Some(s.id);
                        self.notice = format!("Press D again to remove '{}'", s.name);
                    } else if let Err(e) = self.library.remove_sound(s.id) {
                        self.notice = format!("Delete failed: {e}")
                    } else {
                        self.pending_delete = None;
                        self.notice = "Sound removed".into();
                        self.reload();
                        self.configure_hotkeys();
                    }
                }
            }
            KeyCode::Char('b') if self.selected_sound().is_some() => {
                self.pending_hotkey_capture = self.selected_sound().map(|sound| sound.id);
                let _ = self.runtime.commands.send(RuntimeCommand::SuspendHotkeys);
                self.notice = if self.is_chinese() {
                    "正在暂停现有快捷键…"
                } else {
                    "Suspending existing hotkeys…"
                }
                .into();
            }
            KeyCode::Char('e')
                if self.focus == Focus::Sounds && self.selected_sound().is_some() =>
            {
                self.input = Some(
                    self.selected_sound()
                        .map(|sound| sound.name)
                        .unwrap_or_default(),
                );
                self.notice = "Edit sound name: Enter saves".into();
            }
            KeyCode::Char('f') if self.focus == Focus::Sounds => self.toggle_favorite(),
            KeyCode::Char('g') if self.focus == Focus::Sounds => self.toggle_loop(),
            KeyCode::Char('i') if self.focus == Focus::Sounds => self.cycle_playback_mode(),
            KeyCode::Char('k') if self.focus == Focus::Sounds => self.clear_hotkey(),
            KeyCode::Char('t')
                if self.focus == Focus::Sounds && self.selected_sound().is_some() =>
            {
                self.input = Some(
                    self.selected_sound()
                        .map(|sound| sound.tags)
                        .unwrap_or_default(),
                );
                self.notice = "Edit tags (comma-separated): Enter saves".into();
            }
            KeyCode::Char('v')
                if self.focus == Focus::Sounds && self.selected_sound().is_some() =>
            {
                self.input = Some(
                    self.selected_sound()
                        .map(|sound| sound.note)
                        .unwrap_or_default(),
                );
                self.notice = "Edit note: Enter saves".into();
            }
            KeyCode::Char('x') if self.focus == Focus::Sounds => self.assign_current_category(),
            KeyCode::Char('n') if self.focus == Focus::Categories => {
                self.input = Some(String::new());
                self.notice = "New category: type a name then Enter".into();
            }
            KeyCode::Char('r') if self.focus == Focus::Categories && self.selected_category > 2 => {
                self.input = Some(self.categories[self.selected_category - 3].name.clone());
                self.notice = "Rename category: Enter saves".into();
            }
            KeyCode::Char('r') => self.refresh_devices(),
            KeyCode::Char('+') => self.adjust(0.05),
            KeyCode::Char('-') => self.adjust(-0.05),
            KeyCode::Char('m') => self.open_devices(DeviceKind::Mic),
            KeyCode::Char('o') => self.open_devices(DeviceKind::Output),
            KeyCode::Char('l') => self.open_devices(DeviceKind::Monitor),
            _ => {}
        }
    }
    fn finish_input(&mut self, text: String) {
        if self.notice.starts_with("Search:") {
            self.query = text;
            self.reload();
            self.notice = format!("Search: {}", self.query)
        } else if self.notice.starts_with("Command:") {
            self.execute_command(&text);
        } else if self.notice.starts_with("New profile:") {
            match self.library.add_profile(&text, "") {
                Ok(id) => {
                    self.profiles = self.library.profiles().unwrap_or_default();
                    self.active_profile_id = id;
                    let _ = self.library.set_active_profile(id);
                    self.config.app.active_profile = text.clone();
                    self.save();
                    self.category = None;
                    self.special_filter = None;
                    self.selected_category = 0;
                    self.reload();
                    self.configure_hotkeys();
                    self.notice = format!("Profile created: {text}");
                }
                Err(error) => self.notice = format!("Profile not saved: {error}"),
            }
        } else if self.notice.starts_with("New category:") {
            match self
                .library
                .add_category(&text, Some(self.active_profile_id))
            {
                Ok(_) => {
                    self.notice = "Category created".into();
                    self.reload();
                }
                Err(error) => self.notice = format!("Category not saved: {error}"),
            }
        } else if self.notice.starts_with("Rename category:") {
            if let Some(category) = self
                .categories
                .get(self.selected_category.saturating_sub(3))
            {
                match self.library.rename_category(category.id, &text) {
                    Ok(()) => {
                        self.notice = "Category renamed".into();
                        self.reload();
                    }
                    Err(error) => self.notice = format!("Category not saved: {error}"),
                }
            }
        } else if self.notice.starts_with("Edit sound name:") {
            if let Some(mut sound) = self.selected_sound() {
                sound.name = text;
                match self.library.update_sound(&sound) {
                    Ok(()) => {
                        self.notice = "Sound updated".into();
                        self.reload();
                    }
                    Err(error) => self.notice = format!("Sound not saved: {error}"),
                }
            }
        } else if self.notice.starts_with("Edit tags") {
            self.update_selected(|sound| sound.tags = text, "Tags updated");
        } else if self.notice.starts_with("Edit note:") {
            self.update_selected(|sound| sound.note = text, "Note updated");
        } else if self.notice.starts_with("Add sound:") {
            let name = std::path::Path::new(&text)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("New sound")
                .to_owned();
            let sound = Sound {
                id: 0,
                name,
                file_path: text,
                category_id: self.category,
                profile_id: Some(self.active_profile_id),
                volume: 0.8,
                playback_mode: Default::default(),
                loop_enabled: false,
                favorite: false,
                tags: String::new(),
                note: String::new(),
                sort_order: 0,
                play_count: 0,
                last_played_at: None,
            };
            match self.library.add_sound(sound) {
                Ok(_) => {
                    self.notice = "Sound added".into();
                    self.reload()
                }
                Err(e) => self.notice = format!("Import failed: {e}"),
            }
        }
    }
    fn capture_hotkey_key(&mut self, sound_id: i64, key: KeyEvent) {
        if key.code == KeyCode::Esc {
            self.capture_hotkey = None;
            self.pending_hotkey_capture = None;
            self.configure_hotkeys();
            self.notice = if self.is_chinese() {
                "已取消快捷键绑定"
            } else {
                "Hotkey binding cancelled"
            }
            .into();
            return;
        }
        #[cfg(target_os = "macos")]
        {
            // The global rdev listener preserves Command/Ctrl and three-key
            // modifiers; terminal events (especially VS Code's) do not.
            let _ = (sound_id, key);
        }
        #[cfg(not(target_os = "macos"))]
        self.apply_captured_hotkey_from_event(sound_id, key);
    }
    #[cfg(not(target_os = "macos"))]
    fn apply_captured_hotkey_from_event(&mut self, sound_id: i64, key: KeyEvent) {
        let Some(hotkey) = format_hotkey(key) else {
            self.notice = if self.is_chinese() {
                "该按键暂不支持，请使用字母、数字、方向键或空格"
            } else {
                "Unsupported key; use a letter, number, arrow key, or Space"
            }
            .into();
            return;
        };
        self.capture_hotkey = None;
        self.apply_captured_hotkey(sound_id, hotkey);
    }
    fn apply_captured_hotkey(&mut self, sound_id: i64, hotkey: String) {
        match self.library.set_hotkey(sound_id, &hotkey) {
            Ok(()) => {
                self.notice = if self.is_chinese() {
                    format!("快捷键已绑定：{hotkey}")
                } else {
                    format!("Bound {hotkey}")
                };
                self.configure_hotkeys();
            }
            Err(error) => {
                self.notice = if self.is_chinese() {
                    format!("快捷键绑定失败：{error}")
                } else {
                    format!("Hotkey not saved: {error}")
                }
            }
        }
    }
    fn execute_command(&mut self, command: &str) {
        let mut pieces = command
            .trim()
            .trim_start_matches(':')
            .splitn(3, char::is_whitespace);
        let Some(action) = pieces.next().filter(|value| !value.is_empty()) else {
            self.notice = "Empty command".into();
            return;
        };
        match action {
            "play" => match pieces
                .next()
                .and_then(|id| id.parse::<i64>().ok())
                .and_then(|id| self.sounds.iter().find(|sound| sound.id == id).cloned())
            {
                Some(sound) => {
                    let _ = self
                        .runtime
                        .commands
                        .send(RuntimeCommand::PlaySound(sound.clone()));
                    let _ = self.library.record_play(sound.id, "command");
                    self.notice = format!("Playing {}", sound.name);
                }
                None => self.notice = "Usage: :play <sound id in current list>".into(),
            },
            "stop-all" => {
                let _ = self.runtime.commands.send(RuntimeCommand::StopAll);
                self.playing = None;
                self.notice = "Stopped all sounds".into();
            }
            "set" => {
                let target = pieces.next();
                let value = pieces
                    .next()
                    .map(str::trim)
                    .filter(|value| !value.is_empty());
                match (target, value) {
                    (Some("mic"), Some(value)) => {
                        self.config.audio.devices.mic = Some(value.into());
                        self.save();
                        self.notice = "Microphone configured".into();
                    }
                    (Some("output"), Some(value)) => {
                        self.config.audio.devices.output = Some(value.into());
                        self.save();
                        self.notice = "Output configured".into();
                    }
                    (Some("monitor"), Some(value)) => {
                        self.config.audio.devices.monitor = Some(value.into());
                        self.save();
                        self.notice = "Monitor configured".into();
                    }
                    _ => self.notice = "Usage: :set <mic|output|monitor> <device name>".into(),
                }
            }
            "profile" => match pieces.next().and_then(|name| {
                self.profiles
                    .iter()
                    .find(|profile| profile.name.eq_ignore_ascii_case(name))
                    .cloned()
            }) {
                Some(profile) => match self.library.set_active_profile(profile.id) {
                    Ok(()) => {
                        self.active_profile_id = profile.id;
                        self.config.app.active_profile = profile.name.clone();
                        self.category = None;
                        self.special_filter = None;
                        self.reload();
                        self.configure_hotkeys();
                        self.save();
                        self.notice = format!("Active profile: {}", profile.name);
                    }
                    Err(error) => self.notice = format!("Profile switch failed: {error}"),
                },
                None => self.notice = "Unknown profile".into(),
            },
            "help" => {
                self.page = Page::Help;
            }
            _ => self.notice = format!("Unknown command: {action}"),
        }
    }
    fn adjust(&mut self, delta: f32) {
        if let Some(mut s) = self.selected_sound() {
            s.volume = (s.volume + delta).clamp(0., 1.);
            let _ = self.library.update_sound(&s);
            self.reload();
        }
    }
    fn update_selected(&mut self, update: impl FnOnce(&mut Sound), success: &str) {
        if let Some(mut sound) = self.selected_sound() {
            update(&mut sound);
            match self.library.update_sound(&sound) {
                Ok(()) => {
                    self.notice = success.into();
                    self.reload();
                }
                Err(error) => self.notice = format!("Sound not saved: {error}"),
            }
        }
    }
    fn toggle_favorite(&mut self) {
        self.update_selected(|sound| sound.favorite = !sound.favorite, "Favorite updated");
    }
    fn toggle_loop(&mut self) {
        self.update_selected(
            |sound| sound.loop_enabled = !sound.loop_enabled,
            "Loop setting updated",
        );
    }
    fn cycle_playback_mode(&mut self) {
        self.update_selected(
            |sound| {
                sound.playback_mode = match sound.playback_mode {
                    gamesound_core::PlaybackMode::Overlay => {
                        gamesound_core::PlaybackMode::Interrupt
                    }
                    gamesound_core::PlaybackMode::Interrupt => gamesound_core::PlaybackMode::Queue,
                    gamesound_core::PlaybackMode::Queue => gamesound_core::PlaybackMode::Exclusive,
                    gamesound_core::PlaybackMode::Exclusive => {
                        gamesound_core::PlaybackMode::Overlay
                    }
                }
            },
            "Playback mode updated",
        );
    }
    fn clear_hotkey(&mut self) {
        if let Some(sound) = self.selected_sound() {
            match self.library.clear_hotkey(sound.id) {
                Ok(()) => {
                    self.notice = "Hotkey cleared".into();
                    self.configure_hotkeys();
                }
                Err(error) => self.notice = format!("Hotkey not cleared: {error}"),
            }
        }
    }
    fn assign_current_category(&mut self) {
        let category = self.category;
        self.update_selected(|sound| sound.category_id = category, "Category assigned");
    }
    fn config_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.page = Page::Main,
            KeyCode::Char('m') => self.open_devices(DeviceKind::Mic),
            KeyCode::Char('o') => self.open_devices(DeviceKind::Output),
            KeyCode::Char('l') => self.open_devices(DeviceKind::Monitor),
            KeyCode::Char('1') => {
                self.config.audio.devices.mic = None;
                let _ = self.runtime.commands.send(RuntimeCommand::StopAudio);
                self.save();
                self.notice = if self.is_chinese() {
                    "已清空麦克风设备"
                } else {
                    "Microphone device cleared"
                }
                .into();
            }
            KeyCode::Char('2') => {
                self.config.audio.devices.output = None;
                let _ = self.runtime.commands.send(RuntimeCommand::StopAudio);
                self.save();
                self.notice = if self.is_chinese() {
                    "已清空虚拟输出设备"
                } else {
                    "Virtual output device cleared"
                }
                .into();
            }
            KeyCode::Char('3') => {
                self.config.audio.devices.monitor = None;
                let _ = self.runtime.commands.send(RuntimeCommand::StopAudio);
                self.save();
                self.notice = if self.is_chinese() {
                    "已清空监听设备"
                } else {
                    "Monitor device cleared"
                }
                .into();
            }
            KeyCode::Char('z') => {
                self.config.tui.language = match self.config.tui.language {
                    Language::English => Language::Chinese,
                    Language::Chinese => Language::English,
                };
                self.save();
                self.notice = if self.is_chinese() {
                    "界面语言已切换为中文"
                } else {
                    "Interface language changed to English"
                }
                .into();
            }
            KeyCode::Char('p') => {
                self.profiles = self.library.profiles().unwrap_or_default();
                self.selected_profile = self
                    .profiles
                    .iter()
                    .position(|profile| profile.id == self.active_profile_id)
                    .unwrap_or(0);
                self.page = Page::ProfilePicker;
            }
            KeyCode::Char('n') => {
                self.input = Some(String::new());
                self.notice = "New profile: type a name then Enter".into();
            }
            KeyCode::Char('s') => self.save(),
            KeyCode::Char('t') => self.start_audio(),
            KeyCode::Char('h') => {
                self.config.monitor.enabled = !self.config.monitor.enabled;
                self.save();
                self.notice = format!(
                    "Local monitor {}",
                    if self.config.monitor.enabled {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
            }
            KeyCode::Char('b') => {
                self.config.monitor.mode = match self.config.monitor.mode {
                    MonitorMode::SfxOnly => MonitorMode::FullMix,
                    MonitorMode::FullMix => MonitorMode::SfxOnly,
                };
                self.save();
                self.notice = "Monitor mode updated (restart audio to apply)".into();
            }
            KeyCode::Char('d') => {
                self.config.ducking.enabled = !self.config.ducking.enabled;
                self.apply_ducking();
                self.save();
                self.notice = format!(
                    "Ducking {}",
                    if self.config.ducking.enabled {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
            }
            KeyCode::Char('u') => self.adjust_global_volume(VolumeTarget::Mic, 0.05),
            KeyCode::Char('y') => self.adjust_global_volume(VolumeTarget::Mic, -0.05),
            KeyCode::Char('+') => {
                self.adjust_global_volume(VolumeTarget::Sfx, 0.05);
            }
            KeyCode::Char('-') => {
                self.adjust_global_volume(VolumeTarget::Sfx, -0.05);
            }
            KeyCode::Char(']') => self.adjust_global_volume(VolumeTarget::Monitor, 0.05),
            KeyCode::Char('[') => self.adjust_global_volume(VolumeTarget::Monitor, -0.05),
            _ => {}
        }
    }
    fn adjust_global_volume(&mut self, target: VolumeTarget, delta: f32) {
        let value = match target {
            VolumeTarget::Mic => {
                self.config.volume.mic = (self.config.volume.mic + delta).clamp(0.0, 1.0);
                self.config.volume.mic
            }
            VolumeTarget::Sfx => {
                self.config.volume.sfx = (self.config.volume.sfx + delta).clamp(0.0, 1.0);
                self.config.volume.sfx
            }
            VolumeTarget::Monitor => {
                self.config.volume.monitor = (self.config.volume.monitor + delta).clamp(0.0, 1.0);
                self.config.volume.monitor
            }
        };
        let _ = self
            .runtime
            .commands
            .send(RuntimeCommand::SetVolume { target, value });
        self.save();
    }
    fn profile_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.page = Page::Config,
            KeyCode::Up => self.selected_profile = self.selected_profile.saturating_sub(1),
            KeyCode::Down => {
                self.selected_profile =
                    (self.selected_profile + 1).min(self.profiles.len().saturating_sub(1))
            }
            KeyCode::Enter => {
                if let Some(profile) = self.profiles.get(self.selected_profile).cloned() {
                    match self.library.set_active_profile(profile.id) {
                        Ok(()) => {
                            self.active_profile_id = profile.id;
                            self.config.app.active_profile = profile.name.clone();
                            self.category = None;
                            self.special_filter = None;
                            self.selected_category = 0;
                            self.save();
                            self.reload();
                            self.configure_hotkeys();
                            self.notice = format!("Active profile: {}", profile.name);
                            self.page = Page::Config;
                        }
                        Err(error) => self.notice = format!("Profile switch failed: {error}"),
                    }
                }
            }
            _ => {}
        }
    }
    fn switch_to_next_profile(&mut self) {
        if self.profiles.is_empty() {
            return;
        }
        let current = self
            .profiles
            .iter()
            .position(|p| p.id == self.active_profile_id)
            .unwrap_or(0);
        let profile = self.profiles[(current + 1) % self.profiles.len()].clone();
        if let Err(error) = self.library.set_active_profile(profile.id) {
            self.notice = format!("Profile switch failed: {error}");
            return;
        }
        self.active_profile_id = profile.id;
        self.config.app.active_profile = profile.name.clone();
        self.category = None;
        self.special_filter = None;
        self.reload();
        self.configure_hotkeys();
        self.save();
        self.notice = format!("Active profile: {}", profile.name);
    }
    fn open_devices(&mut self, kind: DeviceKind) {
        self.devices = match kind {
            DeviceKind::Mic => device::input_devices(),
            _ => device::output_devices(),
        }
        .unwrap_or_else(|e| {
            self.notice = format!("Device enumeration failed: {e}");
            vec![]
        });
        self.selected_device = 0;
        self.page = Page::Device(kind)
    }
    fn refresh_devices(&mut self) {
        match (device::input_devices(), device::output_devices()) {
            (Ok(inputs), Ok(outputs)) => {
                self.notice = format!(
                    "Refreshed devices: {} input, {} output",
                    inputs.len(),
                    outputs.len()
                )
            }
            (Err(error), _) | (_, Err(error)) => {
                self.notice = format!("Device refresh failed: {error}")
            }
        }
    }
    fn device_key(&mut self, kind: DeviceKind, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.page = Page::Config,
            KeyCode::Up => self.selected_device = self.selected_device.saturating_sub(1),
            KeyCode::Down => {
                self.selected_device =
                    (self.selected_device + 1).min(self.devices.len().saturating_sub(1))
            }
            KeyCode::Enter => {
                if let Some(d) = self.devices.get(self.selected_device) {
                    match kind {
                        DeviceKind::Mic => self.config.audio.devices.mic = Some(d.id.clone()),
                        DeviceKind::Output => {
                            self.config.audio.devices.output = Some(d.id.clone());
                            if !d.is_virtual {
                                self.notice =
                                    "Warning: selected output does not look like a virtual device"
                                        .into();
                            }
                        }
                        DeviceKind::Monitor => {
                            self.config.audio.devices.monitor = Some(d.id.clone())
                        }
                    }
                    self.save();
                    if self.config.audio.devices.output.is_some() {
                        self.start_audio();
                    }
                    self.page = Page::Config;
                }
            }
            _ => {}
        }
    }
    fn start_audio(&mut self) {
        match self.config.audio.devices.output.clone() {
            Some(output) => {
                self.apply_runtime_volumes();
                self.apply_ducking();
                let _ = self.runtime.commands.send(RuntimeCommand::Start {
                    mic: self.config.audio.devices.mic.clone(),
                    output,
                    monitor: if self.config.monitor.enabled {
                        self.config.audio.devices.monitor.clone()
                    } else {
                        None
                    },
                    monitor_sfx_only: matches!(self.config.monitor.mode, MonitorMode::SfxOnly),
                    preferences: StreamPreferences {
                        sample_rate: self.config.audio.sample_rate,
                        channels: self.config.audio.channels,
                        buffer_size: self.config.audio.buffer_size,
                    },
                });
                if self.config.app.first_run {
                    self.config.app.first_run = false;
                    self.save();
                }
                self.notice = "Starting audio engine…".into()
            }
            None => self.notice = "Choose a virtual output device first (O)".into(),
        }
    }
    fn apply_runtime_volumes(&self) {
        for (target, value) in [
            (VolumeTarget::Mic, self.config.volume.mic),
            (VolumeTarget::Sfx, self.config.volume.sfx),
            (VolumeTarget::Monitor, self.config.volume.monitor),
        ] {
            let _ = self
                .runtime
                .commands
                .send(RuntimeCommand::SetVolume { target, value });
        }
    }
    fn apply_ducking(&self) {
        let _ = self.runtime.commands.send(RuntimeCommand::SetDucking {
            enabled: self.config.ducking.enabled,
            ratio: self.config.ducking.ratio,
            attack_ms: self.config.ducking.attack_ms,
            release_ms: self.config.ducking.release_ms,
            release_delay_ms: self.config.ducking.release_delay_ms,
        });
    }

    fn configure_hotkeys(&mut self) {
        if !self.config.hotkeys.enabled {
            return;
        }
        let bindings = self
            .library
            .sounds_in_profile(self.active_profile_id, None, "")
            .unwrap_or_default()
            .iter()
            .filter_map(|sound| {
                self.library
                    .hotkey(sound.id)
                    .ok()
                    .flatten()
                    .map(|key| (key, sound.clone()))
            })
            .collect();
        let _ = self.runtime.commands.send(RuntimeCommand::SetHotkeys {
            sounds: bindings,
            stop_all: self.config.hotkeys.stop_all.clone(),
            toggle_mic: self.config.hotkeys.toggle_mic.clone(),
            toggle_sfx: self.config.hotkeys.toggle_sfx.clone(),
            toggle_monitor: self.config.hotkeys.toggle_monitor.clone(),
            sfx_volume_up: self.config.hotkeys.sfx_volume_up.clone(),
            sfx_volume_down: self.config.hotkeys.sfx_volume_down.clone(),
            mic_volume_up: self.config.hotkeys.mic_volume_up.clone(),
            mic_volume_down: self.config.hotkeys.mic_volume_down.clone(),
            monitor_volume_up: self.config.hotkeys.monitor_volume_up.clone(),
            monitor_volume_down: self.config.hotkeys.monitor_volume_down.clone(),
            switch_profile: self.config.hotkeys.switch_profile.clone(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn formats_captured_hotkeys() {
        assert_eq!(
            format_hotkey(KeyEvent::new(
                KeyCode::Char('K'),
                KeyModifiers::CONTROL | KeyModifiers::SHIFT
            ))
            .as_deref(),
            Some("ctrl+shift+k")
        );
        assert_eq!(
            format_hotkey(KeyEvent::new(KeyCode::F(5), KeyModifiers::CONTROL)).as_deref(),
            Some("ctrl+f5")
        );
        assert_eq!(
            format_hotkey(KeyEvent::new(KeyCode::Left, KeyModifiers::ALT)).as_deref(),
            Some("alt+left")
        );
        assert!(format_hotkey(KeyEvent::new(KeyCode::Enter, KeyModifiers::empty())).is_none());
    }
}
