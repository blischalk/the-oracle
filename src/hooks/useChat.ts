import { useRef, useEffect, useCallback } from "react";
import { useCampaignStore } from "../stores/campaignStore";
import { useSettingsStore } from "../stores/settingsStore";
import { useNarration } from "./useNarration";

export function useChat() {
  const {
    messages,
    isSending,
    sendMessage,
    error,
    activeCampaignId,
    isRequestingGreeting,
    hasMoreMessages,
    isLoadingOlderMessages,
    loadOlderMessages,
  } = useCampaignStore();
  const { settings } = useSettingsStore();
  const { speak, stop } = useNarration();
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const lastUserMessageRef = useRef<HTMLDivElement>(null);
  const bottomSpacerRef = useRef<HTMLDivElement>(null);
  const topSentinelRef = useRef<HTMLDivElement>(null);
  const lastMessageCountRef = useRef(0);
  const prevCampaignIdRef = useRef<string | null>(null);
  const prevIsRequestingGreetingRef = useRef(false);
  // Guards the top sentinel from firing before the initial scroll-to-bottom runs.
  const readyToLoadOlderRef = useRef(false);

  // When the active campaign changes, scroll to the bottom of the loaded history.
  useEffect(() => {
    if (!activeCampaignId || activeCampaignId === prevCampaignIdRef.current) return;
    prevCampaignIdRef.current = activeCampaignId;
    readyToLoadOlderRef.current = false;
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        const container = scrollContainerRef.current;
        if (container) container.scrollTop = container.scrollHeight;
        readyToLoadOlderRef.current = true;
      });
    });
  }, [activeCampaignId]);

  // Scroll to the top when the GM greeting arrives for a new campaign,
  // so the player reads from the beginning of the message.
  useEffect(() => {
    const greetingJustArrived = prevIsRequestingGreetingRef.current && !isRequestingGreeting;
    prevIsRequestingGreetingRef.current = isRequestingGreeting;
    if (!greetingJustArrived) return;
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        const container = scrollContainerRef.current;
        if (container) container.scrollTop = 0;
      });
    });
  }, [isRequestingGreeting]);

  // Scroll the user's message to the top when sent so the GM response appears below.
  // Also narrate new GM responses when narration is enabled.
  useEffect(() => {
    const count = messages.length;
    if (count === 0) return;
    const lastMessage = messages[count - 1];
    const prevCount = lastMessageCountRef.current;
    lastMessageCountRef.current = count;

    const gmJustResponded = lastMessage.role === "assistant" && count > prevCount;
    if (gmJustResponded) speak(lastMessage.content);

    const userJustSent = lastMessage.role === "user" && count > prevCount;
    if (!userJustSent) return;

    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        const container = scrollContainerRef.current;
        const userMsg = lastUserMessageRef.current;
        const spacer = bottomSpacerRef.current;
        if (!container || !userMsg) return;

        if (spacer) spacer.style.height = `${container.clientHeight}px`;

        const containerTop = container.getBoundingClientRect().top;
        const msgTop = userMsg.getBoundingClientRect().top;
        const delta = msgTop - containerTop;
        if (delta > 0) {
          container.scrollTo({ top: container.scrollTop + delta, behavior: "smooth" });
        }
      });
    });
  }, [messages, speak]);

  // Watch the top sentinel — when it becomes visible and there are older messages,
  // load them while preserving the current scroll position.
  useEffect(() => {
    const sentinel = topSentinelRef.current;
    if (!sentinel) return;

    const observer = new IntersectionObserver(
      (entries) => {
        if (!entries[0].isIntersecting) return;
        if (!readyToLoadOlderRef.current) return;
        if (!hasMoreMessages || isLoadingOlderMessages) return;

        const container = scrollContainerRef.current;
        const heightBefore = container?.scrollHeight ?? 0;

        loadOlderMessages().then(() => {
          requestAnimationFrame(() => {
            if (container) {
              container.scrollTop = container.scrollHeight - heightBefore;
            }
          });
        });
      },
      { root: scrollContainerRef.current, threshold: 0 }
    );

    observer.observe(sentinel);
    return () => observer.disconnect();
  }, [hasMoreMessages, isLoadingOlderMessages, loadOlderMessages]);

  const submit = useCallback(
    (content: string) => {
      stop();
      sendMessage(content, settings.active_provider_id, settings.active_model_id);
    },
    [stop, sendMessage, settings.active_provider_id, settings.active_model_id]
  );

  const scrollToMessage = useCallback((messageId: string) => {
    const container = scrollContainerRef.current;
    const el = document.getElementById(`msg-${messageId}`);
    if (!container || !el) return;
    const containerTop = container.getBoundingClientRect().top;
    const elTop = el.getBoundingClientRect().top;
    container.scrollTo({ top: container.scrollTop + (elTop - containerTop), behavior: "smooth" });
  }, []);

  return {
    messages,
    isSending,
    submit,
    error,
    scrollContainerRef,
    lastUserMessageRef,
    bottomSpacerRef,
    topSentinelRef,
    hasMoreMessages,
    isLoadingOlderMessages,
    scrollToMessage,
  };
}
