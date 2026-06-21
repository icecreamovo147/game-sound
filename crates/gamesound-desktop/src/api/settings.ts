import { invoke } from "@tauri-apps/api/core";
import type { DesktopSettings, ProfileInfo } from "../types";

export async function getSettings(): Promise<DesktopSettings> {
  return invoke("get_settings");
}

export async function updateSettings(params: {
  theme?: string;
  language?: string;
  log_level?: string;
  monitor_mode?: string;
  hotkeys_enabled?: boolean;
}): Promise<void> {
  return invoke("update_settings", { params });
}

export async function exportConfig(): Promise<string> {
  return invoke("export_config");
}

export async function importConfig(tomlContent: string): Promise<void> {
  return invoke("import_config", { tomlContent });
}

export async function resetConfig(): Promise<void> {
  return invoke("reset_config");
}

export async function openConfigDir(): Promise<void> {
  return invoke("open_config_dir");
}

export async function openLogDir(): Promise<void> {
  return invoke("open_log_dir");
}

export async function getProfileInfo(): Promise<ProfileInfo> {
  return invoke("get_profile_info");
}
