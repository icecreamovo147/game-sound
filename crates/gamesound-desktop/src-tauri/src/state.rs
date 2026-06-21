use anyhow::{Context, Result};
use gamesound_core::{
    audio::StreamPreferences,
    mixer::MixerSettings,
    runtime::{RuntimeHandle, RuntimeStatus},
};
use gamesound_storage::{config::ConfigStore, db::Library};
use std::sync::Mutex;

pub struct AppState {
    pub store: ConfigStore,
    pub library: Mutex<Library>,
    pub runtime: Mutex<Option<RuntimeHandle>>,
    pub runtime_status: Mutex<RuntimeStatus>,
    pub mixer_settings: Mutex<MixerSettings>,
}

impl AppState {
    pub fn new() -> Result<Self> {
        let store =
            ConfigStore::for_current_user().context("cannot locate user config directory")?;
        store.initialise()?;
        let db_path = store.db_path();
        let library = Library::open(&db_path).context("cannot open sound library database")?;
        Ok(Self {
            store,
            library: Mutex::new(library),
            runtime: Mutex::new(None),
            runtime_status: Mutex::new(RuntimeStatus::Stopped),
            mixer_settings: Mutex::new(MixerSettings::default()),
        })
    }

    pub fn restore_config(&self) -> Result<()> {
        let config = self.store.load()?;
        let mut settings = self.mixer_settings.lock().unwrap();
        settings.mic_volume = config.volume.mic;
        settings.sfx_volume = config.volume.sfx;
        settings.monitor_volume = config.volume.monitor;
        settings.ducking = config.ducking.enabled;
        settings.duck_ratio = config.ducking.ratio;
        settings.duck_attack_ms = config.ducking.attack_ms;
        settings.duck_release_ms = config.ducking.release_ms;
        settings.duck_release_delay_ms = config.ducking.release_delay_ms;
        Ok(())
    }

    pub fn restore_hotkeys(&self) -> Result<()> {
        // Hotkeys are re-registered when the engine starts via configure_hotkeys()
        Ok(())
    }

    pub fn save_config(&self) -> Result<()> {
        let mut config = self.store.load().unwrap_or_default();
        let settings = self.mixer_settings.lock().unwrap();
        config.volume.mic = settings.mic_volume;
        config.volume.sfx = settings.sfx_volume;
        config.volume.monitor = settings.monitor_volume;
        config.ducking.enabled = settings.ducking;
        config.ducking.ratio = settings.duck_ratio;
        config.ducking.attack_ms = settings.duck_attack_ms;
        config.ducking.release_ms = settings.duck_release_ms;
        config.ducking.release_delay_ms = settings.duck_release_delay_ms;
        self.store.save(&config)?;
        Ok(())
    }

    pub fn stream_preferences(&self) -> StreamPreferences {
        let config = self.store.load().unwrap_or_default();
        StreamPreferences {
            sample_rate: config.audio.sample_rate,
            channels: config.audio.channels,
            buffer_size: config.audio.buffer_size,
        }
    }

    pub fn configure_hotkeys(&self, handle: &RuntimeHandle) -> Result<usize> {
        use gamesound_core::runtime::RuntimeCommand;

        let config = self.store.load().unwrap_or_default();
        let library = self.library.lock().unwrap();
        let active_profile = library.active_profile().ok();
        let profile_id = active_profile.as_ref().map(|p| p.id);

        // Collect sound hotkeys
        let sounds = if let Some(pid) = profile_id {
            library.sounds_in_profile(pid, None, "").unwrap_or_default()
        } else {
            library.sounds(None, "").unwrap_or_default()
        };

        let mut sound_bindings: Vec<(String, gamesound_core::Sound)> = Vec::new();
        for sound in &sounds {
            if let Ok(Some(hotkey)) = library.hotkey(sound.id) {
                sound_bindings.push((hotkey, sound.clone()));
            }
        }

        let count = sound_bindings.len();
        let _ = handle.commands.send(RuntimeCommand::SetHotkeys {
            sounds: sound_bindings,
            stop_all: config.hotkeys.stop_all.clone(),
            toggle_mic: config.hotkeys.toggle_mic.clone(),
            toggle_sfx: config.hotkeys.toggle_sfx.clone(),
            toggle_monitor: config.hotkeys.toggle_monitor.clone(),
            sfx_volume_up: config.hotkeys.sfx_volume_up.clone(),
            sfx_volume_down: config.hotkeys.sfx_volume_down.clone(),
            mic_volume_up: config.hotkeys.mic_volume_up.clone(),
            mic_volume_down: config.hotkeys.mic_volume_down.clone(),
            monitor_volume_up: config.hotkeys.monitor_volume_up.clone(),
            monitor_volume_down: config.hotkeys.monitor_volume_down.clone(),
            switch_profile: config.hotkeys.switch_profile.clone(),
        });
        Ok(count)
    }
}
