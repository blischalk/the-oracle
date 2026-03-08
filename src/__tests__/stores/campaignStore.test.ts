import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

// Import store AFTER mocking
const { useCampaignStore } = await import("../../stores/campaignStore");

describe("campaignStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useCampaignStore.setState({ campaigns: [], activeCampaignId: null, messages: [], isSending: false, error: null });
  });

  it("loads campaigns", async () => {
    const fakeCampaigns = [{ id: "1", name: "Test", rpg_system_id: "cairn", created_at: "", updated_at: "", is_archived: false }];
    mockedInvoke.mockResolvedValueOnce(fakeCampaigns);
    await useCampaignStore.getState().loadCampaigns();
    expect(useCampaignStore.getState().campaigns).toEqual(fakeCampaigns);
  });

  it("sets error on failed load", async () => {
    mockedInvoke.mockRejectedValueOnce(new Error("DB error"));
    await useCampaignStore.getState().loadCampaigns();
    expect(useCampaignStore.getState().error).toBeTruthy();
  });

  it("selects campaign and loads messages", async () => {
    const fakeMessages = [{ id: "m1", campaign_id: "1", role: "user", content: "hello", created_at: "" }];
    mockedInvoke.mockResolvedValueOnce(fakeMessages);
    await useCampaignStore.getState().selectCampaign("1");
    expect(useCampaignStore.getState().activeCampaignId).toBe("1");
    expect(useCampaignStore.getState().messages).toEqual(fakeMessages);
  });
});
