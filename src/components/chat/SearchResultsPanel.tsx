import type { Message } from "../../domain/campaign";

interface SearchResult {
  message: Message;
  matchingLines: string[];
}

function findMatchingLines(content: string, query: string): string[] {
  const lower = query.toLowerCase();
  return content
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.toLowerCase().includes(lower));
}

function HighlightedLine({ line, query }: { line: string; query: string }) {
  const lower = line.toLowerCase();
  const lowerQuery = query.toLowerCase();
  const parts: { text: string; highlighted: boolean }[] = [];
  let pos = 0;

  while (pos < line.length) {
    const idx = lower.indexOf(lowerQuery, pos);
    if (idx === -1) {
      parts.push({ text: line.slice(pos), highlighted: false });
      break;
    }
    if (idx > pos) parts.push({ text: line.slice(pos, idx), highlighted: false });
    parts.push({ text: line.slice(idx, idx + query.length), highlighted: true });
    pos = idx + query.length;
  }

  return (
    <span>
      {parts.map((part, i) =>
        part.highlighted ? (
          <mark
            key={i}
            style={{
              backgroundColor: "var(--color-primary)",
              color: "var(--color-bg)",
              borderRadius: "2px",
              padding: "0 2px",
              fontWeight: 600,
            }}
          >
            {part.text}
          </mark>
        ) : (
          <span key={i}>{part.text}</span>
        )
      )}
    </span>
  );
}

interface Props {
  messages: Message[];
  query: string;
  onSelectMessage: (messageId: string) => void;
}

export function SearchResultsPanel({ messages, query, onSelectMessage }: Props) {
  const results: SearchResult[] = messages
    .map((message) => ({
      message,
      matchingLines: findMatchingLines(message.content, query),
    }))
    .filter((r) => r.matchingLines.length > 0);

  if (results.length === 0) {
    return (
      <div
        className="flex-1 flex items-center justify-center"
        style={{ color: "var(--color-text-muted)", fontFamily: "var(--font-body)", fontSize: "0.9375rem" }}
      >
        No messages match "{query}"
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto" style={{ padding: "var(--space-3)" }}>
      <div style={{ color: "var(--color-text-muted)", fontSize: "0.8125rem", marginBottom: "var(--space-3)", fontFamily: "var(--font-body)" }}>
        {results.length} message{results.length !== 1 ? "s" : ""} matched
      </div>
      {results.map(({ message, matchingLines }) => (
        <button
          key={message.id}
          type="button"
          onClick={() => onSelectMessage(message.id)}
          style={{
            display: "block",
            width: "100%",
            textAlign: "left",
            background: "none",
            border: "1px solid var(--color-border)",
            borderRadius: "var(--border-radius)",
            padding: "var(--space-2) var(--space-3)",
            marginBottom: "var(--space-2)",
            cursor: "pointer",
            fontFamily: "var(--font-body)",
          }}
          onMouseEnter={(e) => (e.currentTarget.style.borderColor = "var(--color-primary)")}
          onMouseLeave={(e) => (e.currentTarget.style.borderColor = "var(--color-border)")}
        >
          <div style={{ fontSize: "0.75rem", color: "var(--color-text-muted)", marginBottom: "4px", textTransform: "uppercase", letterSpacing: "0.04em" }}>
            {message.role === "user" ? "You" : "GM"}
          </div>
          <div style={{ fontSize: "0.875rem", color: "var(--color-text)", lineHeight: "1.5" }}>
            {matchingLines.map((line, i) => (
              <div key={i} style={{ marginBottom: matchingLines.length > 1 ? "2px" : 0 }}>
                <HighlightedLine line={line} query={query} />
              </div>
            ))}
          </div>
        </button>
      ))}
    </div>
  );
}
