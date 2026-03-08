import { describe, it, expect, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const { exportCampaignToMarkdown } = await import("../../services/campaignService");

describe("exportCampaignToMarkdown", () => {
  const campaign = {
    id: "c1",
    name: "The Blighted Wood",
    rpg_system_id: "cairn",
    created_at: "2024-01-01T00:00:00Z",
    updated_at: "2024-01-01T00:00:00Z",
    is_archived: false,
  };

  it("includes campaign name in header", () => {
    const md = exportCampaignToMarkdown(campaign, []);
    expect(md).toContain("# The Blighted Wood");
  });

  it("includes rpg system in header", () => {
    const md = exportCampaignToMarkdown(campaign, []);
    expect(md).toContain("cairn");
  });

  it("renders user messages as bold You", () => {
    const messages = [{ id: "m1", campaign_id: "c1", role: "user" as const, content: "I look around.", created_at: "" }];
    const md = exportCampaignToMarkdown(campaign, messages);
    expect(md).toContain("**You**");
    expect(md).toContain("I look around.");
  });

  it("renders assistant messages as bold The Oracle", () => {
    const messages = [{ id: "m2", campaign_id: "c1", role: "assistant" as const, content: "The woods are dark.", created_at: "" }];
    const md = exportCampaignToMarkdown(campaign, messages);
    expect(md).toContain("**The Oracle**");
    expect(md).toContain("The woods are dark.");
  });

  it("excludes system messages", () => {
    const messages = [
      { id: "m0", campaign_id: "c1", role: "system" as const, content: "You are a GM.", created_at: "" },
      { id: "m1", campaign_id: "c1", role: "user" as const, content: "Hello.", created_at: "" },
    ];
    const md = exportCampaignToMarkdown(campaign, messages);
    expect(md).not.toContain("You are a GM.");
    expect(md).toContain("Hello.");
  });
});
