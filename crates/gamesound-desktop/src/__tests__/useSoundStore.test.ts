import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { useSoundStore } from "../stores/useSoundStore";

const mockInvoke = invoke as ReturnType<typeof vi.fn>;

describe("useSoundStore", () => {
  beforeEach(() => {
    useSoundStore.setState({
      sounds: [],
      categories: [],
      selectedCategory: null,
      searchQuery: "",
      isLoading: false,
      error: null,
    });
    mockInvoke.mockReset();
  });

  it("fetchSounds loads sounds with category and search filters", async () => {
    const mockSounds = [
      { id: 1, name: "boom", file_path: "/sounds/boom.mp3", category_id: null, volume: 0.8, playback_mode: "overlay", loop_enabled: false, favorite: false, tags: "", note: "", sort_order: 0, play_count: 0, last_played_at: null, hotkey: null },
      { id: 2, name: "laugh", file_path: "/sounds/laugh.mp3", category_id: 1, volume: 0.9, playback_mode: "overlay", loop_enabled: false, favorite: true, tags: "", note: "", sort_order: 0, play_count: 5, last_played_at: null, hotkey: "ctrl+1" },
    ];
    mockInvoke.mockResolvedValueOnce(mockSounds);

    useSoundStore.setState({ selectedCategory: 1, searchQuery: "boom" });
    await useSoundStore.getState().fetchSounds();

    const state = useSoundStore.getState();
    expect(state.sounds).toEqual(mockSounds);
    expect(mockInvoke).toHaveBeenCalledWith("list_sounds", { category: 1, query: "boom" });
    expect(state.isLoading).toBe(false);
  });

  it("fetchCategories loads categories", async () => {
    const mockCategories = [
      { id: 1, name: "Memes", profile_id: null, sort_order: 0 },
      { id: 2, name: "Voice lines", profile_id: null, sort_order: 1 },
    ];
    mockInvoke.mockResolvedValueOnce(mockCategories);

    await useSoundStore.getState().fetchCategories();

    expect(useSoundStore.getState().categories).toEqual(mockCategories);
  });

  it("setSelectedCategory triggers fetchSounds", async () => {
    mockInvoke.mockResolvedValueOnce([]);
    const store = useSoundStore.getState();

    store.setSelectedCategory(5);

    // Should have triggered fetchSounds with the new category
    expect(mockInvoke).toHaveBeenCalledWith("list_sounds", { category: 5, query: null });
    expect(useSoundStore.getState().selectedCategory).toBe(5);
  });

  it("addSound triggers fetchSounds after adding", async () => {
    mockInvoke.mockResolvedValueOnce({ id: 3, name: "new", file_path: "/new.mp3", category_id: 1, volume: 0.8, playback_mode: "overlay", loop_enabled: false, favorite: false, tags: "", note: "", sort_order: 0, play_count: 0, last_played_at: null, hotkey: null });
    mockInvoke.mockResolvedValueOnce([]); // fetchSounds

    await useSoundStore.getState().addSound("/new.mp3", "new", 1);

    expect(mockInvoke).toHaveBeenCalledWith("add_sound", {
      filePath: "/new.mp3",
      name: "new",
      categoryId: 1,
    });
  });

  it("deleteSound removes sound and refreshes list", async () => {
    mockInvoke.mockResolvedValueOnce(undefined); // deleteSound
    mockInvoke.mockResolvedValueOnce([]); // fetchSounds

    await useSoundStore.getState().deleteSound(1);

    expect(mockInvoke).toHaveBeenCalledWith("delete_sound", { id: 1 });
  });
});
