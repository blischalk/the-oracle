import ReactMarkdown, { type Components } from "react-markdown";
import remarkGfm from "remark-gfm";
import { visit } from "unist-util-visit";
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

// Remark plugin: splits text nodes around query matches and tags the matching
// spans as a custom "queryMatch" node so the ReactMarkdown component map can
// render them as <mark> without needing rehype-raw.
function createQueryHighlightPlugin(query: string) {
  const lowerQuery = query.toLowerCase();
  return () => (tree: Parameters<typeof visit>[0]) => {
    visit(tree, "text", (node: { type: string; value: string }, index, parent) => {
      if (!parent || index === undefined) return;
      const text: string = node.value;
      const lower = text.toLowerCase();
      const replacement: unknown[] = [];
      let pos = 0;

      while (pos < text.length) {
        const idx = lower.indexOf(lowerQuery, pos);
        if (idx === -1) {
          replacement.push({ type: "text", value: text.slice(pos) });
          break;
        }
        if (idx > pos) {
          replacement.push({ type: "text", value: text.slice(pos, idx) });
        }
        replacement.push({ type: "queryMatch", value: text.slice(idx, idx + query.length) });
        pos = idx + query.length;
      }

      if (replacement.length > 1 || (replacement[0] as { type: string }).type === "queryMatch") {
        (parent as { children: unknown[] }).children.splice(index, 1, ...replacement);
        return index + replacement.length;
      }
    });
  };
}

// Cast is required because react-markdown's Components type only lists known
// HTML tags, but we need to handle the custom "queryMatch" node injected by
// the remark plugin above.
const queryMatchComponents = {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  queryMatch: ({ node }: any) => (
    <mark
      style={{
        backgroundColor: "var(--color-primary)",
        color: "var(--color-bg)",
        borderRadius: "2px",
        padding: "0 2px",
        fontWeight: 600,
      }}
    >
      {node.value}
    </mark>
  ),
} as unknown as Components;

function HighlightedMarkdownLine({ line, query }: { line: string; query: string }) {
  return (
    <ReactMarkdown
      remarkPlugins={[remarkGfm, createQueryHighlightPlugin(query)]}
      components={queryMatchComponents}
    >
      {line}
    </ReactMarkdown>
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
                <HighlightedMarkdownLine line={line} query={query} />
              </div>
            ))}
          </div>
        </button>
      ))}
    </div>
  );
}
