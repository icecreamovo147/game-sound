import { create } from "zustand";
import * as soundsApi from "../api/sounds";
import type { SoundInfo, CategoryInfo } from "../types";

interface SoundState {
  sounds: SoundInfo[];
  categories: CategoryInfo[];
  selectedCategory: number | null;
  searchQuery: string;
  isLoading: boolean;
  error: string | null;
  fetchSounds: () => Promise<void>;
  fetchCategories: () => Promise<void>;
  setSelectedCategory: (id: number | null) => void;
  setSearchQuery: (query: string) => void;
  addSound: (filePath: string, name?: string | null, categoryId?: number | null) => Promise<SoundInfo>;
  updateSound: (params: Parameters<typeof soundsApi.updateSound>[0]) => Promise<void>;
  deleteSound: (id: number) => Promise<void>;
  playSound: (id: number) => Promise<void>;
  stopSound: (id: number) => Promise<void>;
  stopAll: () => Promise<void>;
  addCategory: (name: string) => Promise<void>;
  updateCategory: (id: number, name: string) => Promise<void>;
  deleteCategory: (id: number) => Promise<void>;
}

export const useSoundStore = create<SoundState>((set, get) => ({
  sounds: [],
  categories: [],
  selectedCategory: null,
  searchQuery: "",
  isLoading: false,
  error: null,

  fetchSounds: async () => {
    const { selectedCategory, searchQuery } = get();
    set({ isLoading: true, error: null });
    try {
      const sounds = await soundsApi.listSounds(
        selectedCategory,
        searchQuery || null,
      );
      set({ sounds, isLoading: false });
    } catch (e) {
      set({ error: String(e), isLoading: false });
    }
  },

  fetchCategories: async () => {
    try {
      const categories = await soundsApi.listCategories();
      set({ categories });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  setSelectedCategory: (id) => {
    set({ selectedCategory: id });
    get().fetchSounds();
  },

  setSearchQuery: (query) => {
    set({ searchQuery: query });
  },

  addSound: async (filePath, name, categoryId) => {
    const info = await soundsApi.addSound(filePath, name, categoryId);
    await get().fetchSounds();
    return info;
  },

  updateSound: async (params) => {
    await soundsApi.updateSound(params);
    await get().fetchSounds();
  },

  deleteSound: async (id) => {
    await soundsApi.deleteSound(id);
    await get().fetchSounds();
  },

  playSound: async (id) => {
    await soundsApi.playSound(id);
  },

  stopSound: async (id) => {
    await soundsApi.stopSound(id);
  },

  stopAll: async () => {
    await soundsApi.stopAllSounds();
  },

  addCategory: async (name) => {
    await soundsApi.addCategory(name);
    await get().fetchCategories();
  },

  updateCategory: async (id, name) => {
    await soundsApi.updateCategory(id, name);
    await get().fetchCategories();
  },

  deleteCategory: async (id) => {
    await soundsApi.deleteCategory(id);
    set({ selectedCategory: null });
    await get().fetchCategories();
    await get().fetchSounds();
  },
}));
