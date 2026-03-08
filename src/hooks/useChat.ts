import { useRef, useEffect } from "react";
import { useCampaignStore } from "../stores/campaignStore";
import { useSettingsStore } from "../stores/settingsStore";

export function useChat() {
  const { messages, isSending, sendMessage, error } = useCampaignStore();
  const { settings } = useSettingsStore();
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  function submit(content: string) {
    sendMessage(content, settings.active_provider_id, settings.active_model_id);
  }

  return { messages, isSending, submit, error, bottomRef };
}
