import { invoke } from "@tauri-apps/api/core";
import type { MixerInfo, UpdateMixerParams } from "../types";

export async function getMixerSettings(): Promise<MixerInfo> {
  return invoke("get_mixer_settings");
}

export async function updateMixerSettings(params: UpdateMixerParams): Promise<MixerInfo> {
  return invoke("update_mixer_settings", { params });
}

export async function setMicVolume(value: number): Promise<void> {
  return invoke("set_mic_volume", { value });
}

export async function setSfxVolume(value: number): Promise<void> {
  return invoke("set_sfx_volume", { value });
}

export async function setMonitorVolume(value: number): Promise<void> {
  return invoke("set_monitor_volume", { value });
}

export async function toggleMicMute(): Promise<boolean> {
  return invoke("toggle_mic_mute");
}

export async function toggleSfxMute(): Promise<boolean> {
  return invoke("toggle_sfx_mute");
}

export async function toggleMonitor(): Promise<boolean> {
  return invoke("toggle_monitor");
}
