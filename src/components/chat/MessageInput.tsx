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
      className="flex gap-2 p-4"
      style={{ borderTop: "1px solid var(--color-border)", backgroundColor: "var(--color-surface)" }}
    >
      <textarea
        className="flex-1 resize-none rounded p-2 text-sm focus:outline-none"
        style={{
          backgroundColor: "var(--color-bg)",
          border: "1px solid var(--color-border)",
          color: "var(--color-text)",
          fontFamily: "var(--font-body)",
          borderRadius: "var(--border-radius)",
          minHeight: "2.5rem",
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
        className="px-4 py-2 font-semibold text-sm transition-opacity"
        style={{
          backgroundColor: "var(--color-primary)",
          color: "var(--color-bg)",
          borderRadius: "var(--border-radius)",
          opacity: disabled ? 0.5 : 1,
          cursor: disabled ? "not-allowed" : "pointer",
        }}
        onClick={submit}
        disabled={disabled}
      >
        Send
      </button>
    </div>
  );
}
