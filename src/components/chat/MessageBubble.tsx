import { useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { Message } from "../../domain/campaign";

interface Props {
  message: Message;
  isNew?: boolean;
}

// Each paragraph fades in sequentially. Stagger is compressed for very long
// responses so the total animation never exceeds MAX_TOTAL_MS.
const PARA_STAGGER_MS = 120;
const PARA_FADE_MS = 200;
const MAX_TOTAL_MS = 3000;

interface AnimatedTextProps {
  content: string;
  onComplete: () => void;
}

function AnimatedText({ content, onComplete }: AnimatedTextProps) {
  const paragraphs = content.split(/\n\n+/).filter((p) => p.trim());

  if (paragraphs.length === 0) {
    onComplete();
    return null;
  }

  const staggerMs =
    paragraphs.length * PARA_STAGGER_MS > MAX_TOTAL_MS
      ? Math.floor(MAX_TOTAL_MS / paragraphs.length)
      : PARA_STAGGER_MS;

  return (
    <>
      {paragraphs.map((para, i) => (
        <div
          key={i}
          style={{
            opacity: 0,
            animation: `wordFadeIn ${PARA_FADE_MS}ms both`,
            animationDelay: `${i * staggerMs}ms`,
          }}
          onAnimationEnd={i === paragraphs.length - 1 ? onComplete : undefined}
        >
          <ReactMarkdown remarkPlugins={[remarkGfm]}>{para}</ReactMarkdown>
        </div>
      ))}
    </>
  );
}

const bubbleStyle: React.CSSProperties = {
  padding: "var(--space-3)",
  fontSize: "var(--font-size-body, 1.0625rem)",
  lineHeight: "var(--line-height-body, var(--line-height-relaxed))",
  border: "1px solid var(--color-border)",
  color: "var(--color-text)",
  fontFamily: "var(--font-body)",
  borderRadius: "var(--border-radius)",
};

export function MessageBubble({ message, isNew = false }: Props) {
  const isUser = message.role === "user";
  // Only animate once — on the first render of a new assistant message.
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
