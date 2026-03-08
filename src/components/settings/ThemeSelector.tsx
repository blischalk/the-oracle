import { useTheme } from "../../hooks/useTheme";

export function ThemeSelector() {
  const { themes, activeTheme, setTheme } = useTheme();

  return (
    <div>
      <label className="oracle-label">Theme</label>
      <div className="grid grid-cols-3 gap-2" style={{ marginTop: "var(--space-2)" }}>
        {themes.map((t) => (
          <button
            key={t.id}
            type="button"
            className="text-center transition-all"
            style={{
              padding: "var(--input-padding-y) var(--input-padding-x)",
              fontSize: "0.8125rem",
              lineHeight: "var(--line-height-tight)",
              border: `1px solid ${activeTheme === t.id ? "var(--color-primary)" : "var(--color-border)"}`,
              color: activeTheme === t.id ? "var(--color-primary)" : "var(--color-text-muted)",
              borderRadius: "var(--border-radius)",
              backgroundColor: activeTheme === t.id ? "var(--color-surface)" : "transparent",
              cursor: "pointer",
            }}
            onClick={() => setTheme(t.id)}
          >
            {t.label}
          </button>
        ))}
      </div>
    </div>
  );
}
