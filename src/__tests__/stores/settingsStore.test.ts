import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const mockedInvoke = vi.mocked(invoke);

const { useSettingsStore } = await import("../../stores/settingsStore");

describe("settingsStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("loads settings from backend", async () => {
    const fakeSettings = { active_provider_id: "anthropic", active_model_id: "claude-opus-4-5", theme: "dungeon", is_fullscreen: false };
    mockedInvoke.mockResolvedValueOnce(fakeSettings);
    await useSettingsStore.getState().loadSettings();
    expect(useSettingsStore.getState().settings.theme).toBe("dungeon");
  });
});
