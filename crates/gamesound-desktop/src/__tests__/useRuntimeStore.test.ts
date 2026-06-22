import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock Tauri invoke before importing the store
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import { useRuntimeStore } from "../stores/useRuntimeStore";

const mockInvoke = invoke as ReturnType<typeof vi.fn>;

describe("useRuntimeStore", () => {
  beforeEach(() => {
    useRuntimeStore.setState({
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
    });
    mockInvoke.mockReset();
  });

  it("fetchStatus updates status from backend", async () => {
    mockInvoke.mockResolvedValueOnce({
      status: "Running",
      mic_device: "Test Mic",
      output_device: "BlackHole 2ch",
      monitor_device: null,
      hotkeys_enabled: true,
      active_sounds: [],
    });

    await useRuntimeStore.getState().fetchStatus();

    const state = useRuntimeStore.getState();
    expect(state.status.status).toBe("Running");
    expect(state.status.mic_device).toBe("Test Mic");
    expect(state.status.output_device).toBe("BlackHole 2ch");
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
  });

  it("fetchStatus sets error on failure", async () => {
    mockInvoke.mockRejectedValueOnce("Network error");

    await useRuntimeStore.getState().fetchStatus();

    const state = useRuntimeStore.getState();
    expect(state.error).toBe("Network error");
    expect(state.isLoading).toBe(false);
  });

  it("startEngine sets status to Running on success", async () => {
    mockInvoke.mockResolvedValueOnce(undefined);

    await useRuntimeStore.getState().startEngine();

    const state = useRuntimeStore.getState();
    expect(state.status.status).toBe("Running");
    expect(state.error).toBeNull();
  });

  it("stopEngine sets status to Stopped and clears active sounds", async () => {
    useRuntimeStore.setState({
      status: {
        status: "Running",
        mic_device: "Mic",
        output_device: "Out",
        monitor_device: null,
        hotkeys_enabled: true,
        active_sounds: [1, 2, 3],
      },
    });
    mockInvoke.mockResolvedValueOnce(undefined);

    await useRuntimeStore.getState().stopEngine();

    const state = useRuntimeStore.getState();
    expect(state.status.status).toBe("Stopped");
    expect(state.status.active_sounds).toEqual([]);
  });

  it("setError and setWarning update state", () => {
    useRuntimeStore.getState().setError("Something went wrong");
    expect(useRuntimeStore.getState().error).toBe("Something went wrong");

    useRuntimeStore.getState().setError(null);
    expect(useRuntimeStore.getState().error).toBeNull();

    useRuntimeStore.getState().setWarning("Low disk space");
    expect(useRuntimeStore.getState().warning).toBe("Low disk space");
  });
});
