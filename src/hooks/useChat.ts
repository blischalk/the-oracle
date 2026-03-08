import { useRef, useEffect } from "react";
import { useCampaignStore } from "../stores/campaignStore";
import { useSettingsStore } from "../stores/settingsStore";

export function useChat() {
  const { messages, isSending, sendMessage, error } = useCampaignStore();
  const { settings } = useSettingsStore();
  const bottomRef = useRef<HTMLDivElement>(null);
  const lastMessageCountRef = useRef(0);

  // Only auto-scroll when the user has just sent a message (so they see their message and the
  // start of the reply). Do not scroll when the assistant message arrives—let the user scroll
  // down at their own pace.
  useEffect(() => {
    const count = messages.length;
    if (count === 0) return;
    const lastMessage = messages[count - 1];
    const prevCount = lastMessageCountRef.current;
    lastMessageCountRef.current = count;

    const userJustSent = lastMessage.role === "user" && count > prevCount;
    if (userJustSent) {
      bottomRef.current?.scrollIntoView({ behavior: "smooth" });
    }
  }, [messages]);

  function submit(content: string) {
    sendMessage(content, settings.active_provider_id, settings.active_model_id);
  }

  return { messages, isSending, submit, error, bottomRef };
}
