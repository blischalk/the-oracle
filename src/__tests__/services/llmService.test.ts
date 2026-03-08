import { describe, it, expect, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({
    content: "Hello adventurer!",
    input_tokens: 10,
    output_tokens: 20,
  }),
}));

const { sendChatMessage, listProviders } = await import("../../services/llmService");

describe("llmService", () => {
  it("sendChatMessage calls invoke with correct command", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const result = await sendChatMessage("campaign-1", "hello", "anthropic", "claude-opus-4-5");
    expect(invoke).toHaveBeenCalledWith("send_chat_message", {
      campaignId: "campaign-1",
      userMessage: "hello",
      providerId: "anthropic",
      modelId: "claude-opus-4-5",
    });
    expect(result.content).toBe("Hello adventurer!");
  });

  it("listProviders calls invoke with list_providers", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    vi.mocked(invoke).mockResolvedValueOnce([]);
    await listProviders();
    expect(invoke).toHaveBeenCalledWith("list_providers");
  });
});
