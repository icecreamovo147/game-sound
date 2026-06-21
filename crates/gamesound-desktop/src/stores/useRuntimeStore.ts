import { create } from "zustand";
import * as runtimeApi from "../api/runtime";
import type { RuntimeStatusInfo } from "../types";

interface RuntimeState {
  status: RuntimeStatusInfo;
  isLoading: boolean;
  error: string | null;
  warning: string | null;
  fetchStatus: () => Promise<void>;
  startEngine: () => Promise<void>;
  stopEngine: () => Promise<void>;
  restartEngine: () => Promise<void>;
  updateStatus: (partial: Partial<RuntimeStatusInfo>) => void;
  setError: (error: string | null) => void;
  setWarning: (warning: string | null) => void;
}

export const useRuntimeStore = create<RuntimeState>((set, get) => ({
  status: {
    status: "Stopped",
    mic_device: null,
    output_device: null,
    monitor_device: null,
    hotkeys_enabled: false,
    active_sounds: [],
  },
  isLoading: false,
  error: null,
  warning: null,

  fetchStatus: async () => {
    set({ isLoading: true, error: null });
    try {
      const status = await runtimeApi.getRuntimeStatus();
      set({ status, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  startEngine: async () => {
    set({ isLoading: true, error: null });
    try {
      await runtimeApi.startAudioEngine();
      set({
        status: { ...get().status, status: "Running" },
        isLoading: false,
      });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  stopEngine: async () => {
    set({ isLoading: true, error: null });
    try {
      await runtimeApi.stopAudioEngine();
      set({
        status: { ...get().status, status: "Stopped", active_sounds: [] },
        isLoading: false,
      });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  restartEngine: async () => {
    set({ isLoading: true, error: null });
    try {
      await runtimeApi.restartAudioEngine();
      const status = await runtimeApi.getRuntimeStatus();
      set({ status, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  updateStatus: (partial) =>
    set((s) => ({ status: { ...s.status, ...partial } })),
  setError: (error) => set({ error }),
  setWarning: (warning) => set({ warning }),
}));
