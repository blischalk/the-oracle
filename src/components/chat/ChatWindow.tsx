import { useChat } from "../../hooks/useChat";
import { useCampaignStore } from "../../stores/campaignStore";
import { MessageBubble } from "./MessageBubble";
import { MessageInput } from "./MessageInput";
import { TypingIndicator } from "./TypingIndicator";

export function ChatWindow() {
  const { messages, isSending, submit, bottomRef } = useChat();
  const { activeCampaignId } = useCampaignStore();

  if (!activeCampaignId) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <p style={{ color: "var(--color-text-muted)", fontFamily: "var(--font-body)" }}>
          Select a campaign or create a new one to begin your adventure.
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex-1 overflow-y-auto p-4">
        {messages.map((msg) => (
          <MessageBubble key={msg.id} message={msg} />
        ))}
        {isSending && <TypingIndicator />}
        <div ref={bottomRef} />
      </div>
      <MessageInput onSubmit={submit} disabled={isSending} />
    </div>
  );
}
