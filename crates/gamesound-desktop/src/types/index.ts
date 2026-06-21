// ── Sound types ──

export interface SoundInfo {
  id: number;
  name: string;
  file_path: string;
  category_id: number | null;
  profile_id: number | null;
  volume: number;
  playback_mode: "overlay" | "interrupt" | "queue" | "exclusive";
  loop_enabled: boolean;
  favorite: boolean;
  tags: string;
  note: string;
  sort_order: number;
  play_count: number;
  last_played_at: string | null;
  hotkey: string | null;
}

export interface CategoryInfo {
  id: number;
  name: string;
  profile_id: number | null;
  sort_order: number;
}

// ── Device types ──

export interface DeviceInfo {
  id: string;
  name: string;
  is_virtual: boolean;
  device_type: "input" | "output";
}

export interface DeviceList {
  inputs: DeviceInfo[];
  outputs: DeviceInfo[];
}

// ── Mixer types ──

export interface MixerInfo {
  mic_volume: number;
  sfx_volume: number;
  monitor_volume: number;
  mic_muted: boolean;
  sfx_muted: boolean;
  monitor_muted: boolean;
  ducking_enabled: boolean;
  duck_ratio: number;
  duck_attack_ms: number;
  duck_release_ms: number;
  duck_release_delay_ms: number;
  mic_level: number;
  output_level: number;
  monitor_level: number;
}

export interface UpdateMixerParams {
  mic_volume?: number;
  sfx_volume?: number;
  monitor_volume?: number;
  mic_muted?: boolean;
  sfx_muted?: boolean;
  monitor_muted?: boolean;
  ducking_enabled?: boolean;
  duck_ratio?: number;
  duck_attack_ms?: number;
  duck_release_ms?: number;
  duck_release_delay_ms?: number;
}

// ── Hotkey types ──

export interface HotkeyBinding {
  sound_id: number;
  sound_name: string;
  hotkey: string;
}

// ── Runtime types ──

export interface RuntimeStatusInfo {
  status: "Stopped" | "Running" | "Warning";
  mic_device: string | null;
  output_device: string | null;
  monitor_device: string | null;
  hotkeys_enabled: boolean;
  active_sounds: number[];
}

// ── Settings types ──

export interface DesktopSettings {
  mic_device: string | null;
  output_device: string | null;
  monitor_device: string | null;
  sample_rate: number;
  channels: number;
  buffer_size: number;
  mic_volume: number;
  sfx_volume: number;
  monitor_volume: number;
  monitor_enabled: boolean;
  monitor_mode: "sfx_only" | "full_mix" | "off";
  ducking_enabled: boolean;
  duck_ratio: number;
  duck_attack_ms: number;
  duck_release_ms: number;
  duck_release_delay_ms: number;
  hotkeys_enabled: boolean;
  hotkey_stop_all: string;
  hotkey_toggle_mic: string;
  hotkey_toggle_sfx: string;
  hotkey_toggle_monitor: string;
  theme: string;
  language: string;
  log_level: string;
  config_dir: string;
  log_dir: string;
}

export interface ProfileInfo {
  id: number;
  name: string;
  description: string;
  is_active: boolean;
}

// ── Runtime event types ──

export type RuntimeEvent =
  | { type: "SoundStarted"; data: { id: number } }
  | { type: "SoundStopped"; data: { id: number } }
  | { type: "HotkeysSuspended"; data: null }
  | { type: "HotkeysRegistered"; data: { count: number } }
  | { type: "HotkeyCaptured"; data: { shortcut: string } }
  | { type: "Levels"; data: { mic: number; output: number; monitor: number } }
  | { type: "Status"; data: { status: string } }
  | { type: "Error"; data: { message: string } }
  | { type: "Warning"; data: { message: string } }
  | { type: "SwitchProfileRequested"; data: null };

// ── Update sound params ──

export interface UpdateSoundParams {
  id: number;
  name?: string;
  category_id?: number | null;
  volume?: number;
  playback_mode?: string;
  loop_enabled?: boolean;
  favorite?: boolean;
  tags?: string;
  note?: string;
  sort_order?: number;
}
