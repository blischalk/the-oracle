import { describe, it, expect, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn().mockResolvedValue(undefined) }));

const { getSettings, saveSettings, saveApiKey, getApiKey, deleteApiKey } =
  await import("../../services/settingsService");

describe("settingsService", () => {
  it("getSettings calls get_settings command", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValueOnce({ active_provider_id: "anthropic", active_model_id: "claude-opus-4-5", theme: "default", is_fullscreen: false });
    const settings = await getSettings();
    expect(invoke).toHaveBeenCalledWith("get_settings");
    expect(settings.active_provider_id).toBe("anthropic");
  });

  it("saveSettings calls save_settings command", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const settings = { active_provider_id: "anthropic", active_model_id: "claude-opus-4-5", theme: "dungeon", is_fullscreen: false };
    await saveSettings(settings);
    expect(invoke).toHaveBeenCalledWith("save_settings", { settings });
  });

  it("saveApiKey calls save_api_key command", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    await saveApiKey("anthropic", "sk-test");
    expect(invoke).toHaveBeenCalledWith("save_api_key", { providerId: "anthropic", key: "sk-test" });
  });

  it("getApiKey calls get_api_key command", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValueOnce("sk-stored");
    const key = await getApiKey("anthropic");
    expect(invoke).toHaveBeenCalledWith("get_api_key", { providerId: "anthropic" });
    expect(key).toBe("sk-stored");
  });

  it("deleteApiKey calls delete_api_key command", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    await deleteApiKey("anthropic");
    expect(invoke).toHaveBeenCalledWith("delete_api_key", { providerId: "anthropic" });
  });
});
