use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_virtual: bool,
}
fn virtual_name(name: &str) -> bool {
    let name = name.to_lowercase();
    [
        "blackhole",
        "vb-cable",
        "cable input",
        "voicemeeter",
        "loopback",
        "virtual",
        "pipewire",
        "pulse",
    ]
    .iter()
    .any(|needle| name.contains(needle))
}
pub fn input_devices() -> Result<Vec<AudioDevice>> {
    let host = cpal::default_host();
    host.input_devices()
        .context("cannot enumerate input devices")?
        .map(|d| {
            let name = d.name().unwrap_or_else(|_| "Unknown input".into());
            Ok(AudioDevice {
                id: name.clone(),
                is_virtual: virtual_name(&name),
                name,
            })
        })
        .collect()
}
pub fn output_devices() -> Result<Vec<AudioDevice>> {
    let host = cpal::default_host();
    host.output_devices()
        .context("cannot enumerate output devices")?
        .map(|d| {
            let name = d.name().unwrap_or_else(|_| "Unknown output".into());
            Ok(AudioDevice {
                id: name.clone(),
                is_virtual: virtual_name(&name),
                name,
            })
        })
        .collect()
}
pub fn find_input(id: &str) -> Result<cpal::Device> {
    cpal::default_host()
        .input_devices()?
        .find(|d| d.name().ok().as_deref() == Some(id))
        .context("selected microphone is unavailable")
}
pub fn find_output(id: &str) -> Result<cpal::Device> {
    cpal::default_host()
        .output_devices()?
        .find(|d| d.name().ok().as_deref() == Some(id))
        .context("selected output device is unavailable")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recognizes_common_virtual_devices() {
        assert!(virtual_name("BlackHole 2ch"));
        assert!(!virtual_name("MacBook Speakers"));
    }
}
