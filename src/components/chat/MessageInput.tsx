import { useState, KeyboardEvent } from "react";

interface Props {
  onSubmit: (content: string) => void;
  disabled: boolean;
}

export function MessageInput({ onSubmit, disabled }: Props) {
  const [text, setText] = useState("");

  function handleKeyDown(e: KeyboardEvent<HTMLTextAreaElement>) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      submit();
    }
  }

  function submit() {
    const trimmed = text.trim();
    if (!trimmed || disabled) return;
    onSubmit(trimmed);
    setText("");
  }

  return (
    <div
      className="flex gap-3"
      style={{
        padding: "var(--space-4)",
        borderTop: "1px solid var(--color-border)",
        backgroundColor: "var(--color-surface)",
        alignItems: "flex-end",
      }}
    >
      <textarea
        className="oracle-input flex-1 resize-none min-w-0 focus:outline-none"
        style={{
          backgroundColor: "var(--color-bg)",
          border: "1px solid var(--color-border)",
          color: "var(--color-text)",
          fontFamily: "var(--font-body)",
          minHeight: "2.75rem",
          maxHeight: "8rem",
        }}
        placeholder="What do you do?"
        value={text}
        onChange={(e) => setText(e.target.value)}
        onKeyDown={handleKeyDown}
        disabled={disabled}
        rows={2}
      />
      <button
        type="button"
        className="oracle-btn shrink-0"
        style={{
          backgroundColor: "var(--color-primary)",
          color: "var(--color-bg)",
          opacity: disabled ? 0.5 : 1,
        }}
        onClick={submit}
        disabled={disabled}
      >
        Send
      </button>
    </div>
  );
}
