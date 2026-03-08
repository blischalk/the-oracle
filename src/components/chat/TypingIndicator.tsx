export function TypingIndicator() {
  return (
    <div className="flex justify-start mb-4">
      <div
        className="px-4 py-3 rounded-lg"
        style={{
          backgroundColor: "var(--color-assistant-bubble)",
          border: "1px solid var(--color-border)",
          borderRadius: "var(--border-radius)",
        }}
      >
        <div className="flex gap-1 items-center h-5">
          {[0, 1, 2].map((i) => (
            <span
              key={i}
              className="inline-block w-2 h-2 rounded-full animate-bounce"
              style={{
                backgroundColor: "var(--color-primary)",
                animationDelay: `${i * 150}ms`,
              }}
            />
          ))}
        </div>
      </div>
    </div>
  );
}
