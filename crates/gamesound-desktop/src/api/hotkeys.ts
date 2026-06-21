import { invoke } from "@tauri-apps/api/core";
import type { HotkeyBinding } from "../types";

export async function listHotkeys(): Promise<HotkeyBinding[]> {
  return invoke("list_hotkeys");
}

export async function bindHotkey(soundId: number, hotkey: string): Promise<void> {
  return invoke("bind_hotkey", { soundId, hotkey });
}

export async function unbindHotkey(soundId: number): Promise<void> {
  return invoke("unbind_hotkey", { soundId });
}

export async function enableHotkeys(): Promise<void> {
  return invoke("enable_hotkeys");
}

export async function disableHotkeys(): Promise<void> {
  return invoke("disable_hotkeys");
}

export async function reregisterHotkeys(): Promise<number> {
  return invoke("reregister_hotkeys");
}

export async function startHotkeyCapture(): Promise<void> {
  return invoke("start_hotkey_capture");
}
