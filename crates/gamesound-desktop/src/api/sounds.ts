import { invoke } from "@tauri-apps/api/core";
import type { SoundInfo, CategoryInfo, UpdateSoundParams } from "../types";

export async function listSounds(
  category?: number | null,
  query?: string | null,
): Promise<SoundInfo[]> {
  return invoke("list_sounds", { category, query });
}

export async function addSound(
  filePath: string,
  name?: string | null,
  categoryId?: number | null,
): Promise<SoundInfo> {
  return invoke("add_sound", {
    filePath,
    name,
    categoryId,
  });
}

export async function updateSound(params: UpdateSoundParams): Promise<SoundInfo> {
  return invoke("update_sound", { params });
}

export async function deleteSound(id: number): Promise<void> {
  return invoke("delete_sound", { id });
}

export async function playSound(id: number): Promise<void> {
  return invoke("play_sound", { id });
}

export async function stopSound(id: number): Promise<void> {
  return invoke("stop_sound", { id });
}

export async function stopAllSounds(): Promise<void> {
  return invoke("stop_all_sounds");
}

export async function listCategories(): Promise<CategoryInfo[]> {
  return invoke("list_categories");
}

export async function addCategory(name: string, profileId?: number | null): Promise<CategoryInfo> {
  return invoke("add_category", { name, profileId });
}

export async function updateCategory(id: number, name: string): Promise<void> {
  return invoke("update_category", { id, name });
}

export async function deleteCategory(id: number): Promise<void> {
  return invoke("delete_category", { id });
}
