import { invoke } from "@tauri-apps/api/core";
import type { DeviceList } from "../types";

export async function listAudioDevices(): Promise<DeviceList> {
  return invoke("list_audio_devices");
}

export async function refreshAudioDevices(): Promise<DeviceList> {
  return invoke("refresh_audio_devices");
}

export async function setMicDevice(deviceName: string): Promise<void> {
  return invoke("set_mic_device", { deviceName });
}

export async function setVirtualOutputDevice(deviceName: string): Promise<void> {
  return invoke("set_virtual_output_device", { deviceName });
}

export async function setMonitorDevice(deviceName: string): Promise<void> {
  return invoke("set_monitor_device", { deviceName });
}
