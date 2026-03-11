import { useRef, useEffect, useCallback } from "react";
import { useCampaignStore } from "../stores/campaignStore";
import { useSettingsStore } from "../stores/settingsStore";
import { useNarration } from "./useNarration";

export function useChat() {
  const { messages, isSending, sendMessage, error, activeCampaignId, isRequestingGreeting } = useCampaignStore();
  const { settings } = useSettingsStore();
  const { speak, stop } = useNarration();
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const lastUserMessageRef = useRef<HTMLDivElement>(null);
  const bottomSpacerRef = useRef<HTMLDivElement>(null);
  const lastMessageCountRef = useRef(0);
  const prevCampaignIdRef = useRef<string | null>(null);
  const prevIsRequestingGreetingRef = useRef(false);

  // When the active campaign changes, scroll to the bottom of the loaded history.
  useEffect(() => {
    if (!activeCampaignId || activeCampaignId === prevCampaignIdRef.current) return;
    prevCampaignIdRef.current = activeCampaignId;
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        const container = scrollContainerRef.current;
        if (container) container.scrollTop = container.scrollHeight;
      });
    });
  }, [activeCampaignId]);

  // Scroll to bottom when the GM greeting arrives.
  useEffect(() => {
    const greetingJustArrived = prevIsRequestingGreetingRef.current && !isRequestingGreeting;
    prevIsRequestingGreetingRef.current = isRequestingGreeting;
    if (!greetingJustArrived) return;
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        const container = scrollContainerRef.current;
        if (container) container.scrollTop = container.scrollHeight;
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

  const submit = useCallback(
    (content: string) => {
      stop();
      sendMessage(content, settings.active_provider_id, settings.active_model_id);
    },
    [stop, sendMessage, settings.active_provider_id, settings.active_model_id]
  );

  return { messages, isSending, submit, error, scrollContainerRef, lastUserMessageRef, bottomSpacerRef };
}
