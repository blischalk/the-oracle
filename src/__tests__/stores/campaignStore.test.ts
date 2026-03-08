import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const mockedInvoke = vi.mocked(invoke);

const { useCampaignStore } = await import("../../stores/campaignStore");

const resetState = () =>
  useCampaignStore.setState({
    campaigns: [],
    activeCampaignId: null,
    activeCampaign: null,
    campaignState: null,
    activeRpgSystem: null,
    messages: [],
    isSending: false,
    isRequestingGreeting: false,
    error: null,
  });

describe("campaignStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    resetState();
  });

  it("loads campaigns", async () => {
    const fakeCampaigns = [
      { id: "1", name: "Test", rpg_system_id: "cairn", created_at: "", updated_at: "", is_archived: false },
    ];
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
    const fakeCampaign = { id: "1", name: "Test", rpg_system_id: "cairn", created_at: "", updated_at: "", is_archived: false };
    const fakeMessages = [{ id: "m1", campaign_id: "1", role: "user", content: "hello", created_at: "" }];
    const fakeCampaignState = { campaign_id: "1", character_data: {}, notes: "", updated_at: "" };
    const fakeRpgSystem = { id: "cairn", name: "Cairn", character_fields: [], opening_hooks: [], mood: { suggested_theme: "dungeon" } };

    // selectCampaign: getCampaign, then getMessages + getCampaignState + getRpgSystem in parallel
    mockedInvoke
      .mockResolvedValueOnce(fakeCampaign)     // get_campaign
      .mockResolvedValueOnce(fakeMessages)     // get_messages
      .mockResolvedValueOnce(fakeCampaignState) // get_campaign_state
      .mockResolvedValueOnce(fakeRpgSystem);   // get_rpg_system

    await useCampaignStore.getState().selectCampaign("1");
    expect(useCampaignStore.getState().activeCampaignId).toBe("1");
    expect(useCampaignStore.getState().messages).toEqual(fakeMessages);
  });
});
