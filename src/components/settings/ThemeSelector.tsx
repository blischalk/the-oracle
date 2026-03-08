import { useTheme } from "../../hooks/useTheme";

export function ThemeSelector() {
  const { themes, activeTheme, setTheme } = useTheme();

  return (
    <div>
      <label className="block text-sm font-semibold mb-2" style={{ color: "var(--color-text)" }}>Theme</label>
      <div className="grid grid-cols-3 gap-2">
        {themes.map((t) => (
          <button
            key={t.id}
            className="py-2 px-3 text-xs text-center transition-all"
            style={{
              border: `1px solid ${activeTheme === t.id ? "var(--color-primary)" : "var(--color-border)"}`,
              color: activeTheme === t.id ? "var(--color-primary)" : "var(--color-text-muted)",
              borderRadius: "var(--border-radius)",
              backgroundColor: activeTheme === t.id ? "var(--color-surface)" : "transparent",
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
