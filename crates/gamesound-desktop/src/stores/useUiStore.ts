import { create } from "zustand";
import type { Locale } from "../i18n";

export type ColorScheme = "dark" | "light" | "auto";

const STORAGE_KEY = "gamesound-color-scheme";
const LOCALE_STORAGE_KEY = "gamesound-locale";

function readStoredScheme(): ColorScheme {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === "light" || stored === "dark" || stored === "auto") {
      return stored;
    }
  } catch (_) {
    // localStorage unavailable (e.g., Tauri restrictive mode)
  }
  return "dark";
}

function readStoredLocale(): Locale {
  try {
    const stored = localStorage.getItem(LOCALE_STORAGE_KEY);
    if (stored === "en" || stored === "zh") {
      return stored;
    }
  } catch (_) {
    // ignore
  }
  return "en";
}

function writeStoredLocale(locale: Locale) {
  try {
    localStorage.setItem(LOCALE_STORAGE_KEY, locale);
  } catch (_) {
    // ignore
  }
}

function writeStoredScheme(scheme: ColorScheme) {
  try {
    localStorage.setItem(STORAGE_KEY, scheme);
  } catch (_) {
    // ignore
  }
}

interface UiState {
  sidebarCollapsed: boolean;
  colorScheme: ColorScheme;
  locale: Locale;
  toggleSidebar: () => void;
  setColorScheme: (scheme: ColorScheme) => void;
  setLocale: (locale: Locale) => void;
  hydrateFromStorage: () => void;
}

export const useUiStore = create<UiState>((set, get) => ({
  sidebarCollapsed: false,
  colorScheme: "dark",
  locale: "en",

  toggleSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),

  setColorScheme: (scheme) => {
    writeStoredScheme(scheme);
    set({ colorScheme: scheme });
  },

  setLocale: (locale) => {
    writeStoredLocale(locale);
    set({ locale });
  },

  hydrateFromStorage: () => {
    const stored = readStoredScheme();
    const storedLocale = readStoredLocale();
    const updates: Partial<UiState> = {};
    if (stored !== get().colorScheme) {
      updates.colorScheme = stored;
    }
    if (storedLocale !== get().locale) {
      updates.locale = storedLocale;
    }
    if (Object.keys(updates).length > 0) {
      set(updates);
    }
  },
}));
