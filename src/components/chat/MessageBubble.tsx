import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { Message } from "../../domain/campaign";

interface Props {
  message: Message;
}

export function MessageBubble({ message }: Props) {
  const isUser = message.role === "user";
  return (
    <div className={`flex ${isUser ? "justify-end" : "justify-start"} mb-4`}>
      <div
        className="max-w-[75%] rounded-lg p-3 text-sm leading-relaxed"
        style={{
          backgroundColor: isUser ? "var(--color-user-bubble)" : "var(--color-assistant-bubble)",
          border: "1px solid var(--color-border)",
          color: "var(--color-text)",
          fontFamily: "var(--font-body)",
          borderRadius: "var(--border-radius)",
        }}
      >
        {isUser ? (
          <p style={{ whiteSpace: "pre-wrap" }}>{message.content}</p>
        ) : (
          <ReactMarkdown remarkPlugins={[remarkGfm]}>{message.content}</ReactMarkdown>
        )}
      </div>
    </div>
  );
}
