import { invoke } from "@tauri-apps/api/core";
import type { RuntimeStatusInfo } from "../types";

export async function getRuntimeStatus(): Promise<RuntimeStatusInfo> {
  return invoke("get_runtime_status");
}

export async function startAudioEngine(): Promise<void> {
  return invoke("start_audio_engine");
}

export async function stopAudioEngine(): Promise<void> {
  return invoke("stop_audio_engine");
}

export async function restartAudioEngine(): Promise<void> {
  return invoke("restart_audio_engine");
}
