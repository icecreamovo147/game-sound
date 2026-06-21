use crate::state::AppState;
use gamesound_core::runtime::{RuntimeCommand, VolumeTarget};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerInfo {
    pub mic_volume: f32,
    pub sfx_volume: f32,
    pub monitor_volume: f32,
    pub mic_muted: bool,
    pub sfx_muted: bool,
    pub monitor_muted: bool,
    pub ducking_enabled: bool,
    pub duck_ratio: f32,
    pub duck_attack_ms: u32,
    pub duck_release_ms: u32,
    pub duck_release_delay_ms: u32,
    pub mic_level: f32,
    pub output_level: f32,
    pub monitor_level: f32,
}

#[tauri::command]
pub fn get_mixer_settings(state: State<AppState>) -> Result<MixerInfo, String> {
    let settings = state.mixer_settings.lock().unwrap();
    Ok(MixerInfo {
        mic_volume: settings.mic_volume,
        sfx_volume: settings.sfx_volume,
        monitor_volume: settings.monitor_volume,
        mic_muted: settings.mic_muted,
        sfx_muted: settings.sfx_muted,
        monitor_muted: settings.monitor_muted,
        ducking_enabled: settings.ducking,
        duck_ratio: settings.duck_ratio,
        duck_attack_ms: settings.duck_attack_ms,
        duck_release_ms: settings.duck_release_ms,
        duck_release_delay_ms: settings.duck_release_delay_ms,
        mic_level: 0.0,
        output_level: 0.0,
        monitor_level: 0.0,
    })
}

#[derive(Debug, Deserialize)]
pub struct UpdateMixerParams {
    pub mic_volume: Option<f32>,
    pub sfx_volume: Option<f32>,
    pub monitor_volume: Option<f32>,
    pub mic_muted: Option<bool>,
    pub sfx_muted: Option<bool>,
    pub monitor_muted: Option<bool>,
    pub ducking_enabled: Option<bool>,
    pub duck_ratio: Option<f32>,
    pub duck_attack_ms: Option<u32>,
    pub duck_release_ms: Option<u32>,
    pub duck_release_delay_ms: Option<u32>,
}

#[tauri::command]
pub fn update_mixer_settings(
    state: State<AppState>,
    params: UpdateMixerParams,
) -> Result<MixerInfo, String> {
    let mut settings = state.mixer_settings.lock().unwrap();
    let runtime = state.runtime.lock().unwrap();

    if let Some(val) = params.mic_volume {
        settings.mic_volume = val.clamp(0., 1.);
        if let Some(handle) = runtime.as_ref() {
            let _ = handle.commands.send(RuntimeCommand::SetVolume {
                target: VolumeTarget::Mic,
                value: settings.mic_volume,
            });
        }
    }
    if let Some(val) = params.sfx_volume {
        settings.sfx_volume = val.clamp(0., 1.);
        if let Some(handle) = runtime.as_ref() {
            let _ = handle.commands.send(RuntimeCommand::SetVolume {
                target: VolumeTarget::Sfx,
                value: settings.sfx_volume,
            });
        }
    }
    if let Some(val) = params.monitor_volume {
        settings.monitor_volume = val.clamp(0., 1.);
        if let Some(handle) = runtime.as_ref() {
            let _ = handle.commands.send(RuntimeCommand::SetVolume {
                target: VolumeTarget::Monitor,
                value: settings.monitor_volume,
            });
        }
    }
    if let Some(val) = params.mic_muted {
        settings.mic_muted = val;
        if let Some(handle) = runtime.as_ref() {
            let _ = handle.commands.send(RuntimeCommand::SetMute {
                target: VolumeTarget::Mic,
                muted: val,
            });
        }
    }
    if let Some(val) = params.sfx_muted {
        settings.sfx_muted = val;
        if let Some(handle) = runtime.as_ref() {
            let _ = handle.commands.send(RuntimeCommand::SetMute {
                target: VolumeTarget::Sfx,
                muted: val,
            });
        }
    }
    if let Some(val) = params.monitor_muted {
        settings.monitor_muted = val;
        if let Some(handle) = runtime.as_ref() {
            let _ = handle.commands.send(RuntimeCommand::SetMute {
                target: VolumeTarget::Monitor,
                muted: val,
            });
        }
    }

    let ducking_changed = params.ducking_enabled.is_some()
        || params.duck_ratio.is_some()
        || params.duck_attack_ms.is_some()
        || params.duck_release_ms.is_some()
        || params.duck_release_delay_ms.is_some();

    if let Some(val) = params.ducking_enabled {
        settings.ducking = val;
    }
    if let Some(val) = params.duck_ratio {
        settings.duck_ratio = val.clamp(0., 1.);
    }
    if let Some(val) = params.duck_attack_ms {
        settings.duck_attack_ms = val;
    }
    if let Some(val) = params.duck_release_ms {
        settings.duck_release_ms = val;
    }
    if let Some(val) = params.duck_release_delay_ms {
        settings.duck_release_delay_ms = val;
    }

    if ducking_changed {
        if let Some(handle) = runtime.as_ref() {
            let _ = handle.commands.send(RuntimeCommand::SetDucking {
                enabled: settings.ducking,
                ratio: settings.duck_ratio,
                attack_ms: settings.duck_attack_ms,
                release_ms: settings.duck_release_ms,
                release_delay_ms: settings.duck_release_delay_ms,
            });
        }
    }

    drop(runtime);

    // Save config
    let _ = state.save_config();

    Ok(MixerInfo {
        mic_volume: settings.mic_volume,
        sfx_volume: settings.sfx_volume,
        monitor_volume: settings.monitor_volume,
        mic_muted: settings.mic_muted,
        sfx_muted: settings.sfx_muted,
        monitor_muted: settings.monitor_muted,
        ducking_enabled: settings.ducking,
        duck_ratio: settings.duck_ratio,
        duck_attack_ms: settings.duck_attack_ms,
        duck_release_ms: settings.duck_release_ms,
        duck_release_delay_ms: settings.duck_release_delay_ms,
        mic_level: 0.0,
        output_level: 0.0,
        monitor_level: 0.0,
    })
}

macro_rules! simple_volume_cmd {
    ($name:ident, $target:expr, $field:ident) => {
        #[tauri::command]
        pub fn $name(state: State<AppState>, value: f32) -> Result<(), String> {
            let mut settings = state.mixer_settings.lock().unwrap();
            settings.$field = value.clamp(0., 1.);
            let runtime = state.runtime.lock().unwrap();
            if let Some(handle) = runtime.as_ref() {
                let _ = handle.commands.send(RuntimeCommand::SetVolume {
                    target: $target,
                    value: settings.$field,
                });
            }
            let _ = state.save_config();
            Ok(())
        }
    };
}

simple_volume_cmd!(set_mic_volume, VolumeTarget::Mic, mic_volume);
simple_volume_cmd!(set_sfx_volume, VolumeTarget::Sfx, sfx_volume);
simple_volume_cmd!(set_monitor_volume, VolumeTarget::Monitor, monitor_volume);

macro_rules! simple_mute_cmd {
    ($name:ident, $target:expr, $field:ident) => {
        #[tauri::command]
        pub fn $name(state: State<AppState>) -> Result<bool, String> {
            let mut settings = state.mixer_settings.lock().unwrap();
            settings.$field = !settings.$field;
            let runtime = state.runtime.lock().unwrap();
            if let Some(handle) = runtime.as_ref() {
                let _ = handle.commands.send(RuntimeCommand::SetMute {
                    target: $target,
                    muted: settings.$field,
                });
            }
            let _ = state.save_config();
            Ok(settings.$field)
        }
    };
}

simple_mute_cmd!(toggle_mic_mute, VolumeTarget::Mic, mic_muted);
simple_mute_cmd!(toggle_sfx_mute, VolumeTarget::Sfx, sfx_muted);
simple_mute_cmd!(toggle_monitor, VolumeTarget::Monitor, monitor_muted);
