import { useRef, useEffect } from "react";
import { useCampaignStore } from "../stores/campaignStore";
import { useSettingsStore } from "../stores/settingsStore";

export function useChat() {
  const { messages, isSending, sendMessage, error } = useCampaignStore();
  const { settings } = useSettingsStore();
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const lastUserMessageRef = useRef<HTMLDivElement>(null);
  const bottomSpacerRef = useRef<HTMLDivElement>(null);
  const lastMessageCountRef = useRef(0);

  // When the user sends a message, scroll so their message sits at the top of the
  // scroll container. The GM's response then appears below and the user can read
  // down at their own pace without being yanked away.
  //
  // The spacer div at the bottom of the message list is expanded to the container's
  // full height before scrolling. This increases scrollHeight so that scrollTop can
  // be set high enough to place the user's message at the very top of the viewport —
  // without the spacer, scrollTop is already at its maximum when the user's message
  // is the last element, so any further scroll is silently clamped to zero.
  useEffect(() => {
    const count = messages.length;
    if (count === 0) return;
    const lastMessage = messages[count - 1];
    const prevCount = lastMessageCountRef.current;
    lastMessageCountRef.current = count;

    const userJustSent = lastMessage.role === "user" && count > prevCount;
    if (!userJustSent) return;

    // Wait for the browser to finish painting the new message before measuring.
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        const container = scrollContainerRef.current;
        const userMsg = lastUserMessageRef.current;
        const spacer = bottomSpacerRef.current;
        if (!container || !userMsg) return;

        // Expand spacer so scrollHeight grows enough to allow the scroll target.
        if (spacer) spacer.style.height = `${container.clientHeight}px`;

        const containerTop = container.getBoundingClientRect().top;
        const msgTop = userMsg.getBoundingClientRect().top;
        const delta = msgTop - containerTop;
        if (delta > 0) {
          container.scrollTo({ top: container.scrollTop + delta, behavior: "smooth" });
        }
      });
    });
  }, [messages]);

  function submit(content: string) {
    sendMessage(content, settings.active_provider_id, settings.active_model_id);
  }

  return { messages, isSending, submit, error, scrollContainerRef, lastUserMessageRef, bottomSpacerRef };
}
