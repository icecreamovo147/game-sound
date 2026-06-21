import { create } from "zustand";
import * as mixerApi from "../api/mixer";
import type { MixerInfo, UpdateMixerParams } from "../types";

interface MixerState {
  settings: MixerInfo;
  isLoading: boolean;
  error: string | null;
  fetchSettings: () => Promise<void>;
  updateSettings: (params: UpdateMixerParams) => Promise<void>;
  setMicVolume: (value: number) => Promise<void>;
  setSfxVolume: (value: number) => Promise<void>;
  setMonitorVolume: (value: number) => Promise<void>;
  toggleMicMute: () => Promise<void>;
  toggleSfxMute: () => Promise<void>;
  toggleMonitor: () => Promise<void>;
  updateLevels: (mic: number, output: number, monitor: number) => void;
}

const defaultMixer: MixerInfo = {
  mic_volume: 0.9,
  sfx_volume: 0.8,
  monitor_volume: 0.6,
  mic_muted: false,
  sfx_muted: false,
  monitor_muted: false,
  ducking_enabled: true,
  duck_ratio: 0.4,
  duck_attack_ms: 50,
  duck_release_ms: 300,
  duck_release_delay_ms: 200,
  mic_level: 0,
  output_level: 0,
  monitor_level: 0,
};

export const useMixerStore = create<MixerState>((set, get) => ({
  settings: { ...defaultMixer },
  isLoading: false,
  error: null,

  fetchSettings: async () => {
    set({ isLoading: true, error: null });
    try {
      const settings = await mixerApi.getMixerSettings();
      set({ settings, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  updateSettings: async (params) => {
    const settings = await mixerApi.updateMixerSettings(params);
    set({ settings });
  },

  setMicVolume: async (value) => {
    await mixerApi.setMicVolume(value);
    set((s) => ({ settings: { ...s.settings, mic_volume: value } }));
  },

  setSfxVolume: async (value) => {
    await mixerApi.setSfxVolume(value);
    set((s) => ({ settings: { ...s.settings, sfx_volume: value } }));
  },

  setMonitorVolume: async (value) => {
    await mixerApi.setMonitorVolume(value);
    set((s) => ({ settings: { ...s.settings, monitor_volume: value } }));
  },

  toggleMicMute: async () => {
    const muted = await mixerApi.toggleMicMute();
    set((s) => ({ settings: { ...s.settings, mic_muted: muted } }));
  },

  toggleSfxMute: async () => {
    const muted = await mixerApi.toggleSfxMute();
    set((s) => ({ settings: { ...s.settings, sfx_muted: muted } }));
  },

  toggleMonitor: async () => {
    const muted = await mixerApi.toggleMonitor();
    set((s) => ({ settings: { ...s.settings, monitor_muted: muted } }));
  },

  updateLevels: (mic, output, monitor) =>
    set((s) => ({
      settings: {
        ...s.settings,
        mic_level: mic,
        output_level: output,
        monitor_level: monitor,
      },
    })),
}));
