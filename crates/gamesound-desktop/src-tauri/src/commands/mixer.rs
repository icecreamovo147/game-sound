use crate::state::AppState;
use gamesound_core::runtime::{RuntimeCommand, VolumeTarget};
use serde::{Deserialize, Serialize};
use tauri::State;

macro_rules! cmd_trace {
    ($name:expr) => {
        tracing::info!(target: "gamesound_desktop::command", "called: {}", $name);
    };
    ($name:expr, success) => {
        tracing::debug!(target: "gamesound_desktop::command", "success: {}", $name);
    };
}

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
    cmd_trace!("get_mixer_settings");

    let settings = state.mixer_settings.lock().unwrap();

    tracing::debug!(
        target: "gamesound_desktop::mixer",
        mic_vol = settings.mic_volume,
        sfx_vol = settings.sfx_volume,
        monitor_vol = settings.monitor_volume,
        ducking = settings.ducking,
        "mixer settings read"
    );
    cmd_trace!("get_mixer_settings", success);

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
    cmd_trace!("update_mixer_settings");

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
        tracing::info!(target: "gamesound_desktop::mixer", muted = val, "mic mute toggled");
    }
    if let Some(val) = params.sfx_muted {
        settings.sfx_muted = val;
        if let Some(handle) = runtime.as_ref() {
            let _ = handle.commands.send(RuntimeCommand::SetMute {
                target: VolumeTarget::Sfx,
                muted: val,
            });
        }
        tracing::info!(target: "gamesound_desktop::mixer", muted = val, "sfx mute toggled");
    }
    if let Some(val) = params.monitor_muted {
        settings.monitor_muted = val;
        if let Some(handle) = runtime.as_ref() {
            let _ = handle.commands.send(RuntimeCommand::SetMute {
                target: VolumeTarget::Monitor,
                muted: val,
            });
        }
        tracing::info!(target: "gamesound_desktop::mixer", muted = val, "monitor mute toggled");
    }

    let ducking_changed = params.ducking_enabled.is_some()
        || params.duck_ratio.is_some()
        || params.duck_attack_ms.is_some()
        || params.duck_release_ms.is_some()
        || params.duck_release_delay_ms.is_some();

    if let Some(val) = params.ducking_enabled {
        settings.ducking = val;
        tracing::info!(target: "gamesound_desktop::mixer", enabled = val, "ducking toggled");
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
        tracing::debug!(
            target: "gamesound_desktop::mixer",
            enabled = settings.ducking,
            ratio = settings.duck_ratio,
            "ducking settings updated"
        );
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
    let _ = state.save_config_with(&settings);
    cmd_trace!("update_mixer_settings", success);

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
    ($name:ident, $target:expr, $field:ident, $label:expr) => {
        #[tauri::command]
        pub fn $name(state: State<AppState>, value: f32) -> Result<(), String> {
            cmd_trace!(stringify!($name));
            let clamped = value.clamp(0., 1.);
            tracing::info!(
                target: "gamesound_desktop::mixer",
                target = $label,
                value = clamped,
                "volume set"
            );

            let mut settings = state.mixer_settings.lock().unwrap();
            settings.$field = clamped;
            let runtime = state.runtime.lock().unwrap();
            if let Some(handle) = runtime.as_ref() {
                let _ = handle.commands.send(RuntimeCommand::SetVolume {
                    target: $target,
                    value: settings.$field,
                });
            }
            let _ = state.save_config_with(&settings);
            cmd_trace!(stringify!($name), success);
            Ok(())
        }
    };
}

simple_volume_cmd!(set_mic_volume, VolumeTarget::Mic, mic_volume, "mic");
simple_volume_cmd!(set_sfx_volume, VolumeTarget::Sfx, sfx_volume, "sfx");
simple_volume_cmd!(
    set_monitor_volume,
    VolumeTarget::Monitor,
    monitor_volume,
    "monitor"
);

macro_rules! simple_mute_cmd {
    ($name:ident, $target:expr, $field:ident, $label:expr) => {
        #[tauri::command]
        pub fn $name(state: State<AppState>) -> Result<bool, String> {
            cmd_trace!(stringify!($name));

            let mut settings = state.mixer_settings.lock().unwrap();
            settings.$field = !settings.$field;

            tracing::info!(
                target: "gamesound_desktop::mixer",
                target = $label,
                muted = settings.$field,
                "mute toggled"
            );

            let runtime = state.runtime.lock().unwrap();
            if let Some(handle) = runtime.as_ref() {
                let _ = handle.commands.send(RuntimeCommand::SetMute {
                    target: $target,
                    muted: settings.$field,
                });
            }
            let _ = state.save_config_with(&settings);
            cmd_trace!(stringify!($name), success);
            Ok(settings.$field)
        }
    };
}

simple_mute_cmd!(toggle_mic_mute, VolumeTarget::Mic, mic_muted, "mic");
simple_mute_cmd!(toggle_sfx_mute, VolumeTarget::Sfx, sfx_muted, "sfx");
simple_mute_cmd!(
    toggle_monitor,
    VolumeTarget::Monitor,
    monitor_muted,
    "monitor"
);
