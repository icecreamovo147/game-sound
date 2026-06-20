//! Deterministic PCM mixing, deliberately isolated from device callbacks.
#[derive(Debug, Clone, Copy, Default)]
pub struct Levels {
    pub mic: f32,
    pub output: f32,
    pub monitor: f32,
}
#[derive(Debug, Clone, Copy)]
pub struct MixerSettings {
    pub mic_volume: f32,
    pub sfx_volume: f32,
    pub monitor_volume: f32,
    pub mic_muted: bool,
    pub sfx_muted: bool,
    pub monitor_muted: bool,
    pub ducking: bool,
    pub duck_ratio: f32,
    pub duck_attack_ms: u32,
    pub duck_release_ms: u32,
    pub duck_release_delay_ms: u32,
}
impl Default for MixerSettings {
    fn default() -> Self {
        Self {
            mic_volume: 0.9,
            sfx_volume: 0.8,
            monitor_volume: 0.6,
            mic_muted: false,
            sfx_muted: false,
            monitor_muted: false,
            ducking: true,
            duck_ratio: 0.4,
            duck_attack_ms: 50,
            duck_release_ms: 300,
            duck_release_delay_ms: 200,
        }
    }
}
pub fn mix_frame(mic: &[f32], effects: &[Vec<f32>], settings: MixerSettings) -> (Vec<f32>, Levels) {
    let active = effects.iter().any(|f| f.iter().any(|x| *x != 0.0));
    let mic_gain = if settings.mic_muted {
        0.0
    } else {
        settings.mic_volume
            * if active && settings.ducking {
                settings.duck_ratio
            } else {
                1.0
            }
    };
    let mut output = Vec::with_capacity(mic.len());
    let mut mic_peak = 0.0f32;
    let mut out_peak = 0.0f32;
    for i in 0..mic.len() {
        let m = mic.get(i).copied().unwrap_or(0.0);
        mic_peak = mic_peak.max(m.abs());
        let s: f32 = if settings.sfx_muted {
            0.0
        } else {
            effects
                .iter()
                .map(|f| f.get(i).copied().unwrap_or(0.0))
                .sum::<f32>()
                * settings.sfx_volume
        };
        let sample = soft_limit(m * mic_gain + s);
        out_peak = out_peak.max(sample.abs());
        output.push(sample);
    }
    (
        output,
        Levels {
            mic: mic_peak,
            output: out_peak,
            monitor: out_peak * settings.monitor_volume,
        },
    )
}
/// Soft knee prevents the harsh discontinuity of a hard clamp near full scale.
pub fn soft_limit(sample: f32) -> f32 {
    sample.tanh()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn limits_and_ducks() {
        let (mixed, level) = mix_frame(
            &[1.0],
            &[vec![1.0]],
            MixerSettings {
                mic_volume: 1.0,
                sfx_volume: 1.0,
                ducking: true,
                duck_ratio: 0.5,
                ..Default::default()
            },
        );
        assert!(mixed[0] <= 1.0);
        assert_eq!(level.mic, 1.0);
    }
}
