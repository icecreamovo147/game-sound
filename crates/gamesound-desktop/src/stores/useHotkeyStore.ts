import { create } from "zustand";
import * as hotkeysApi from "../api/hotkeys";
import type { HotkeyBinding } from "../types";

interface HotkeyState {
  bindings: HotkeyBinding[];
  isCapturing: boolean;
  capturedShortcut: string | null;
  isLoading: boolean;
  error: string | null;
  fetchBindings: () => Promise<void>;
  bindHotkey: (soundId: number, hotkey: string) => Promise<void>;
  unbindHotkey: (soundId: number) => Promise<void>;
  enableHotkeys: () => Promise<void>;
  disableHotkeys: () => Promise<void>;
  reregisterHotkeys: () => Promise<void>;
  startCapture: () => Promise<void>;
  setCapturedShortcut: (shortcut: string | null) => void;
  setIsCapturing: (capturing: boolean) => void;
}

export const useHotkeyStore = create<HotkeyState>((set, get) => ({
  bindings: [],
  isCapturing: false,
  capturedShortcut: null,
  isLoading: false,
  error: null,

  fetchBindings: async () => {
    set({ isLoading: true, error: null });
    try {
      const bindings = await hotkeysApi.listHotkeys();
      set({ bindings, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  bindHotkey: async (soundId, hotkey) => {
    set({ error: null });
    try {
      await hotkeysApi.bindHotkey(soundId, hotkey);
      await get().fetchBindings();
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  unbindHotkey: async (soundId) => {
    set({ error: null });
    try {
      await hotkeysApi.unbindHotkey(soundId);
      await get().fetchBindings();
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  enableHotkeys: async () => {
    await hotkeysApi.enableHotkeys();
    await get().fetchBindings();
  },

  disableHotkeys: async () => {
    await hotkeysApi.disableHotkeys();
    await get().fetchBindings();
  },

  reregisterHotkeys: async () => {
    set({ error: null });
    try {
      await hotkeysApi.reregisterHotkeys();
      await get().fetchBindings();
    } catch (e) {
      set({ error: String(e) });
    }
  },

  startCapture: async () => {
    set({ isCapturing: true, capturedShortcut: null, error: null });
    try {
      await hotkeysApi.startHotkeyCapture();
    } catch (e) {
      set({ isCapturing: false, error: String(e) });
    }
  },

  setCapturedShortcut: (shortcut) =>
    set({ capturedShortcut: shortcut, isCapturing: false }),
  setIsCapturing: (capturing) => set({ isCapturing: capturing }),
}));
