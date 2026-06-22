import { create } from "zustand";
import * as settingsApi from "../api/settings";
import type { DesktopSettings, ProfileInfo } from "../types";

interface SettingsState {
  settings: DesktopSettings | null;
  profile: ProfileInfo | null;
  isLoading: boolean;
  error: string | null;
  fetchSettings: () => Promise<void>;
  updateSettings: (params: {
    theme?: string;
    language?: string;
    log_level?: string;
    monitor_mode?: string;
    hotkeys_enabled?: boolean;
    close_behavior?: string;
  }) => Promise<void>;
  resetConfig: () => Promise<void>;
  exportConfig: () => Promise<string>;
  importConfig: (toml: string) => Promise<void>;
  openConfigDir: () => Promise<void>;
  openLogDir: () => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  settings: null,
  profile: null,
  isLoading: false,
  error: null,

  fetchSettings: async () => {
    set({ isLoading: true, error: null });
    try {
      const [settings, profile] = await Promise.all([
        settingsApi.getSettings(),
        settingsApi.getProfileInfo(),
      ]);
      set({ settings, profile, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  updateSettings: async (params) => {
    await settingsApi.updateSettings(params);
    // Refresh
    const settings = await settingsApi.getSettings();
    set({ settings });
  },

  resetConfig: async () => {
    await settingsApi.resetConfig();
    const settings = await settingsApi.getSettings();
    set({ settings });
  },

  exportConfig: async () => {
    return settingsApi.exportConfig();
  },

  importConfig: async (toml) => {
    await settingsApi.importConfig(toml);
    const settings = await settingsApi.getSettings();
    set({ settings });
  },

  openConfigDir: () => settingsApi.openConfigDir(),
  openLogDir: () => settingsApi.openLogDir(),
}));
