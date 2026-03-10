import { useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { Message } from "../../domain/campaign";

interface Props {
  message: Message;
  isNew?: boolean;
}

// Strips markdown syntax for the brief animation window so markers like ** don't
// appear as literal characters. The text content is identical; only styling differs.
function stripMarkdown(text: string): string {
  return text
    .replace(/\*\*\*(.+?)\*\*\*/gs, "$1")
    .replace(/\*\*(.+?)\*\*/gs, "$1")
    .replace(/\*(.+?)\*/gs, "$1")
    .replace(/___(.+?)___/gs, "$1")
    .replace(/__(.+?)__/gs, "$1")
    .replace(/_(.+?)_/gs, "$1")
    .replace(/`(.+?)`/gs, "$1")
    .replace(/#{1,6}\s+/gm, "")
    .replace(/^>\s*/gm, "")
    .replace(/^[-*+]\s+/gm, "")
    .replace(/^\d+\.\s+/gm, "")
    .replace(/\[(.+?)\]\(.+?\)/g, "$1");
}

// Each word gets a staggered fade-in. The total animation is compressed so it
// never exceeds MAX_TOTAL_MS — meaning dense responses appear at the same wall-clock
// speed as short ones, just with tighter word spacing.
const BASE_STAGGER_MS = 20;
const WORD_FADE_MS = 150;
const MAX_TOTAL_MS = 2800;

interface AnimatedTextProps {
  content: string;
  onComplete: () => void;
}

function AnimatedText({ content, onComplete }: AnimatedTextProps) {
  const words = stripMarkdown(content).split(/\s+/).filter(Boolean);

  if (words.length === 0) {
    onComplete();
    return null;
  }

  const staggerMs =
    words.length * BASE_STAGGER_MS > MAX_TOTAL_MS
      ? Math.floor(MAX_TOTAL_MS / words.length)
      : BASE_STAGGER_MS;

  return (
    <p style={{ lineHeight: "var(--line-height-relaxed)" }}>
      {words.map((word, i) => (
        <span
          key={i}
          style={{
            opacity: 0,
            animation: `wordFadeIn ${WORD_FADE_MS}ms both`,
            animationDelay: `${i * staggerMs}ms`,
          }}
          onAnimationEnd={i === words.length - 1 ? onComplete : undefined}
        >
          {word}{" "}
        </span>
      ))}
    </p>
  );
}

const bubbleStyle: React.CSSProperties = {
  padding: "var(--space-3)",
  fontSize: "1.0625rem",
  lineHeight: "var(--line-height-relaxed)",
  border: "1px solid var(--color-border)",
  color: "var(--color-text)",
  fontFamily: "var(--font-body)",
  borderRadius: "var(--border-radius)",
};

export function MessageBubble({ message, isNew = false }: Props) {
  const isUser = message.role === "user";
  // Only animate once — on the first render of a new assistant message.
  // useState initializer runs exactly once at mount, so subsequent re-renders
  // with isNew=false don't restart the animation.
  const [isAnimating, setIsAnimating] = useState(() => isNew && !isUser);

  return (
    <div className={`flex ${isUser ? "justify-end" : "justify-start"} mb-4`}>
      <div
        className="max-w-[75%] rounded-lg oracle-prose"
        style={{
          ...bubbleStyle,
          backgroundColor: isUser
            ? "var(--color-user-bubble)"
            : "var(--color-assistant-bubble)",
        }}
      >
        {isUser ? (
          <p style={{ whiteSpace: "pre-wrap" }}>{message.content}</p>
        ) : isAnimating ? (
          <AnimatedText
            content={message.content}
            onComplete={() => setIsAnimating(false)}
          />
        ) : (
          <ReactMarkdown remarkPlugins={[remarkGfm]}>
            {message.content}
          </ReactMarkdown>
        )}
      </div>
    </div>
  );
}
