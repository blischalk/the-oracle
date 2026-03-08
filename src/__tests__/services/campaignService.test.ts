import { describe, it, expect, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

const { listCampaigns } = await import("../../services/campaignService");

describe("campaignService", () => {
  it("calls invoke with list_campaigns", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const result = await listCampaigns();
    expect(invoke).toHaveBeenCalledWith("list_campaigns");
    expect(result).toEqual([]);
  });
});
