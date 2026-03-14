export function TypingIndicator() {
  return (
    <div className="flex justify-start mb-4">
      <div className="flex gap-1 items-center" style={{ padding: "0.5rem 0.25rem" }}>
        {[0, 1, 2].map((i) => (
          <span
            key={i}
            style={{
              display: "inline-block",
              width: "7px",
              height: "7px",
              borderRadius: "50%",
              backgroundColor: "var(--color-text-muted)",
              animation: "typingPulse 1.2s ease-in-out infinite",
              animationDelay: `${i * 0.2}s`,
            }}
          />
        ))}
      </div>
    </div>
  );
}
