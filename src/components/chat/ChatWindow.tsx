import { useEffect, useRef } from "react";
import { useChat } from "../../hooks/useChat";
import { useCampaignStore } from "../../stores/campaignStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { useUiStore } from "../../stores/uiStore";
import { MessageBubble } from "./MessageBubble";
import { MessageInput } from "./MessageInput";
import { TypingIndicator } from "./TypingIndicator";

function hasCharacterName(data: Record<string, unknown> | null | undefined): boolean {
  if (!data || typeof data !== "object") return false;
  const name =
    data.character_name ?? data.name ?? data.CharacterName;
  return typeof name === "string" && name.trim().length > 0;
}

const errorBannerStyle: React.CSSProperties = {
  padding: "var(--space-3) var(--space-4)",
  marginBottom: "var(--space-3)",
  backgroundColor: "#fef2f2",
  color: "#b91c1c",
  borderRadius: "var(--border-radius)",
  fontFamily: "var(--font-body)",
  border: "1px solid #fecaca",
};

export function ChatWindow() {
  const { messages, isSending, submit, bottomRef } = useChat();
  const { searchQuery, setSearchQuery } = useUiStore();
  const filtered = searchQuery
    ? messages.filter((m) => m.content.toLowerCase().includes(searchQuery.toLowerCase()))
    : messages;
  const {
    activeCampaignId,
    campaignState,
    requestGreeting,
    isRequestingGreeting,
    extractCharacterData,
    error,
    clearError,
  } = useCampaignStore();
  const { settings, isLoaded: isSettingsLoaded, loadSettings } = useSettingsStore();
  const extractedForCampaignId = useRef<string | null>(null);

  // Ensure settings are loaded when we have a campaign
  useEffect(() => {
    if (activeCampaignId && !isSettingsLoaded) {
      loadSettings();
    }
  }, [activeCampaignId, isSettingsLoaded, loadSettings]);

  // Request GM greeting as soon as a campaign is active (use current settings, don't wait for load)
  useEffect(() => {
    if (!activeCampaignId) return;
    requestGreeting(settings.active_provider_id, settings.active_model_id);
  }, [activeCampaignId, settings.active_provider_id, settings.active_model_id, requestGreeting]);

  // Reset extraction ref when campaign changes so we can re-extract when returning to a campaign
  useEffect(() => {
    if (!activeCampaignId) extractedForCampaignId.current = null;
  }, [activeCampaignId]);

  // When campaign has messages but no saved character name/stats, extract from conversation once
  useEffect(() => {
    if (
      !activeCampaignId ||
      !isSettingsLoaded ||
      isRequestingGreeting ||
      messages.length === 0 ||
      !settings.active_provider_id ||
      !settings.active_model_id
    )
      return;
    const data = campaignState?.character_data as Record<string, unknown> | undefined;
    if (hasCharacterName(data)) return;
    if (extractedForCampaignId.current === activeCampaignId) return;
    extractedForCampaignId.current = activeCampaignId;
    extractCharacterData(settings.active_provider_id, settings.active_model_id);
  }, [
    activeCampaignId,
    campaignState?.character_data,
    isRequestingGreeting,
    isSettingsLoaded,
    messages.length,
    settings.active_provider_id,
    settings.active_model_id,
    extractCharacterData,
  ]);

  if (!activeCampaignId) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center p-4">
        {error && (
          <div role="alert" style={errorBannerStyle} className="w-full max-w-xl mb-4">
            <span>{error}</span>
            {" "}
            <button
              type="button"
              onClick={clearError}
              style={{ textDecoration: "underline", background: "none", border: "none", color: "inherit", cursor: "pointer" }}
              aria-label="Dismiss error"
            >
              Dismiss
            </button>
          </div>
        )}
        <p style={{ color: "var(--color-text-muted)", fontFamily: "var(--font-body)", fontSize: "1.0625rem" }}>
          Select a campaign or create a new one to begin your adventure.
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {messages.length > 0 && (
        <div
          className="flex items-center gap-2 px-4 py-2 flex-shrink-0"
          style={{ borderBottom: "1px solid var(--color-border)", backgroundColor: "var(--color-surface)" }}
        >
          <input
            className="flex-1 text-sm p-1 focus:outline-none bg-transparent"
            style={{ color: "var(--color-text)", fontFamily: "var(--font-body)" }}
            placeholder="Search messages..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
          {searchQuery && (
            <span className="text-xs" style={{ color: "var(--color-text-muted)" }}>
              {filtered.length} result{filtered.length !== 1 ? "s" : ""}
            </span>
          )}
          {searchQuery && (
            <button
              type="button"
              className="text-xs"
              style={{ color: "var(--color-text-muted)" }}
              onClick={() => setSearchQuery("")}
            >
              ✕
            </button>
          )}
        </div>
      )}
      <div className="flex-1 overflow-y-auto p-4">
        {error && (
          <div role="alert" style={errorBannerStyle}>
            <span>{error}</span>
            {" "}
            <button
              type="button"
              onClick={clearError}
              style={{ textDecoration: "underline", background: "none", border: "none", color: "inherit", cursor: "pointer" }}
              aria-label="Dismiss error"
            >
              Dismiss
            </button>
          </div>
        )}
        {filtered.map((msg) => (
          <MessageBubble key={msg.id} message={msg} />
        ))}
        {(isSending || isRequestingGreeting) && <TypingIndicator />}
        <div ref={bottomRef} />
      </div>
      <MessageInput onSubmit={submit} disabled={isSending || isRequestingGreeting} />
    </div>
  );
}
