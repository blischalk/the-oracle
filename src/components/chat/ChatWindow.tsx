import { useEffect, useRef, useState } from "react";
import { useChat } from "../../hooks/useChat";
import { useCampaignStore } from "../../stores/campaignStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { useUiStore } from "../../stores/uiStore";
import { MessageBubble } from "./MessageBubble";
import { MessageInput } from "./MessageInput";
import { SearchResultsPanel } from "./SearchResultsPanel";
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

// A message is considered "new" if it arrived within the last 8 seconds.
// This distinguishes freshly-received LLM responses (animate them) from the
// historical messages loaded when opening an existing campaign (do not animate).
const NEW_MESSAGE_THRESHOLD_MS = 8000;

function isRecentMessage(msg: { created_at: string }): boolean {
  return Date.now() - new Date(msg.created_at).getTime() < NEW_MESSAGE_THRESHOLD_MS;
}

export function ChatWindow() {
  const {
    messages, isSending, submit, scrollContainerRef, lastUserMessageRef,
    bottomSpacerRef, topSentinelRef, hasMoreMessages, isLoadingOlderMessages,
    scrollToMessage,
  } = useChat();
  const { searchQuery, setSearchQuery } = useUiStore();
  const [searchDraft, setSearchDraft] = useState("");
  const [isScrolledFromBottom, setIsScrolledFromBottom] = useState(false);

  function handleChatScroll(e: React.UIEvent<HTMLDivElement>) {
    const el = e.currentTarget;
    setIsScrolledFromBottom(el.scrollHeight - el.scrollTop - el.clientHeight > 100);
  }

  function scrollToBottom() {
    const lastMsg = messages[messages.length - 1];
    if (lastMsg) scrollToMessage(lastMsg.id);
  }

  function handleSearchSubmit() {
    setSearchQuery(searchDraft.trim());
  }

  function handleClearSearch() {
    setSearchDraft("");
    setSearchQuery("");
  }

  function handleSelectSearchResult(messageId: string) {
    handleClearSearch();
    requestAnimationFrame(() => scrollToMessage(messageId));
  }

  // Determine which assistant message should animate. Only the most recent one
  // qualifies, and only when it was just received (not loaded from history).
  const lastMsg = messages[messages.length - 1];
  const animatingMessageId =
    !searchQuery && lastMsg?.role === "assistant" && isRecentMessage(lastMsg)
      ? lastMsg.id
      : null;
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
  // Track the message count at the time of the last extraction attempt so we only
  // re-extract after new conversation has arrived, not on every render.
  const lastExtractionAtMessageCount = useRef(0);

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

  // Reset extraction counter when campaign changes
  useEffect(() => {
    lastExtractionAtMessageCount.current = 0;
  }, [activeCampaignId]);

  // Re-attempt character extraction after every new GM response until a name is found.
  // Requires at least 4 messages (GM intro + user reply + at least one exchange) so the
  // conversation actually contains the character's name before we attempt extraction.
  useEffect(() => {
    if (
      !activeCampaignId ||
      !isSettingsLoaded ||
      isRequestingGreeting ||
      !settings.active_provider_id ||
      !settings.active_model_id
    )
      return;

    const data = campaignState?.character_data as Record<string, unknown> | undefined;
    if (hasCharacterName(data)) return;

    const lastMessage = messages[messages.length - 1];
    const hasEnoughContext = messages.length >= 4 && lastMessage?.role === "assistant";
    if (!hasEnoughContext) return;

    // Only extract if new messages have arrived since the last attempt
    if (messages.length <= lastExtractionAtMessageCount.current) return;
    lastExtractionAtMessageCount.current = messages.length;

    extractCharacterData(settings.active_provider_id, settings.active_model_id);
  }, [
    activeCampaignId,
    campaignState?.character_data,
    isRequestingGreeting,
    isSettingsLoaded,
    messages,
    settings.active_provider_id,
    settings.active_model_id,
    extractCharacterData,
  ]);

  if (!activeCampaignId) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center" style={{ padding: "var(--space-8, 2rem)" }}>
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
        <p style={{ color: "var(--color-text-muted)", fontFamily: "var(--font-body)", fontSize: "1.0625rem", textAlign: "center", maxWidth: "28rem" }}>
          Select a campaign or create a new one to begin your adventure.
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {messages.length > 0 && (
        <div
          className="flex items-center gap-2 flex-shrink-0"
          style={{ borderBottom: "1px solid var(--color-border)", backgroundColor: "var(--color-surface)", padding: "var(--space-2) var(--space-3)" }}
        >
          <input
            className="flex-1 focus:outline-none bg-transparent"
            style={{
              color: "var(--color-text)",
              fontFamily: "var(--font-body)",
              fontSize: "0.875rem",
              padding: "var(--space-1) var(--space-2)",
              border: "1px solid var(--color-border)",
              borderRadius: "var(--border-radius)",
              background: "var(--color-bg)",
            }}
            placeholder="Search messages..."
            value={searchDraft}
            onChange={(e) => setSearchDraft(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleSearchSubmit()}
          />
          <button
            type="button"
            onClick={handleSearchSubmit}
            style={{
              padding: "var(--space-1) var(--space-2)",
              fontSize: "0.8125rem",
              borderRadius: "var(--border-radius)",
              border: "1px solid var(--color-border)",
              background: "var(--color-primary)",
              color: "var(--color-bg)",
              cursor: "pointer",
              fontFamily: "var(--font-body)",
              flexShrink: 0,
            }}
          >
            Search
          </button>
          {searchQuery && (
            <button
              type="button"
              style={{ fontSize: "0.8125rem", color: "var(--color-text-muted)", background: "none", border: "none", cursor: "pointer", flexShrink: 0 }}
              onClick={handleClearSearch}
              title="Clear search"
            >
              ✕
            </button>
          )}
        </div>
      )}
      <div className="flex-1 overflow-hidden" style={{ position: "relative", display: "flex", flexDirection: "column" }}>
        {searchQuery ? (
          <SearchResultsPanel
            messages={messages}
            query={searchQuery}
            onSelectMessage={handleSelectSearchResult}
          />
        ) : (
          <div ref={scrollContainerRef} className="flex-1 overflow-y-auto p-4" onScroll={handleChatScroll}>
            <div ref={topSentinelRef} aria-hidden />
            {isLoadingOlderMessages && (
              <div style={{ textAlign: "center", padding: "var(--space-3)", color: "var(--color-text-muted)", fontFamily: "var(--font-body)", fontSize: "0.875rem" }}>
                Loading earlier messages…
              </div>
            )}
            {!isLoadingOlderMessages && !hasMoreMessages && messages.length > 0 && (
              <div style={{ textAlign: "center", padding: "var(--space-3)", color: "var(--color-text-muted)", fontFamily: "var(--font-body)", fontSize: "0.8125rem" }}>
                Beginning of conversation
              </div>
            )}
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
            {messages.map((msg, index) => {
              const isLastUserMessage =
                msg.role === "user" &&
                !messages.slice(index + 1).some((m) => m.role === "user");
              return (
                <div key={msg.id} id={`msg-${msg.id}`} ref={isLastUserMessage ? lastUserMessageRef : undefined}>
                  <MessageBubble message={msg} isNew={msg.id === animatingMessageId} />
                </div>
              );
            })}
            {(isSending || isRequestingGreeting) && <TypingIndicator />}
            <div ref={bottomSpacerRef} aria-hidden />
          </div>
        )}
        {isScrolledFromBottom && !searchQuery && !isSending && !isRequestingGreeting && (
          <button
            type="button"
            onClick={scrollToBottom}
            aria-label="Jump to latest message"
            title="Jump to latest message"
            style={{
              position: "absolute",
              bottom: "var(--space-3)",
              right: "var(--space-3)",
              display: "flex",
              alignItems: "center",
              gap: "5px",
              padding: "5px 10px 5px 8px",
              background: "var(--color-surface)",
              color: "var(--color-text-muted)",
              border: "1px solid var(--color-border)",
              borderRadius: "9999px",
              cursor: "pointer",
              fontFamily: "var(--font-body)",
              fontSize: "0.75rem",
              boxShadow: "0 1px 6px rgba(0,0,0,0.15)",
              whiteSpace: "nowrap",
            }}
          >
            <svg width="12" height="12" viewBox="0 0 14 14" fill="none" aria-hidden>
              <path d="M2 4l5 5 5-5" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
            Latest
          </button>
        )}
      </div>
      <MessageInput onSubmit={submit} disabled={isSending || isRequestingGreeting} />
    </div>
  );
}
