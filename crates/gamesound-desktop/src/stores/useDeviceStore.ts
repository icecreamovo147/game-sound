import { create } from "zustand";
import * as devicesApi from "../api/devices";
import type { DeviceInfo } from "../types";

interface DeviceState {
  inputs: DeviceInfo[];
  outputs: DeviceInfo[];
  selectedMic: string | null;
  selectedOutput: string | null;
  selectedMonitor: string | null;
  isLoading: boolean;
  error: string | null;
  fetchDevices: () => Promise<void>;
  setMicDevice: (device: string) => Promise<void>;
  setOutputDevice: (device: string) => Promise<void>;
  setMonitorDevice: (device: string) => Promise<void>;
}

export const useDeviceStore = create<DeviceState>((set, _get) => ({
  inputs: [],
  outputs: [],
  selectedMic: null,
  selectedOutput: null,
  selectedMonitor: null,
  isLoading: false,
  error: null,

  fetchDevices: async () => {
    set({ isLoading: true, error: null });
    try {
      const list = await devicesApi.refreshAudioDevices();
      set({ inputs: list.inputs, outputs: list.outputs, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  setMicDevice: async (device) => {
    await devicesApi.setMicDevice(device);
    set({ selectedMic: device });
  },

  setOutputDevice: async (device) => {
    await devicesApi.setVirtualOutputDevice(device);
    set({ selectedOutput: device });
  },

  setMonitorDevice: async (device) => {
    await devicesApi.setMonitorDevice(device);
    set({ selectedMonitor: device });
  },
}));
