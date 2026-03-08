import { useUiStore } from "../../stores/uiStore";
import { LlmProviderSelector } from "./LlmProviderSelector";
import { ThemeSelector } from "./ThemeSelector";

export function SettingsPanel() {
  const { closeSettings } = useUiStore();

  return (
    <div className="fixed inset-0 flex items-center justify-center z-50" style={{ backgroundColor: "rgba(0,0,0,0.7)" }}>
      <div
        className="w-[480px] max-h-[80vh] overflow-y-auto rounded-lg"
        style={{
          backgroundColor: "var(--color-surface)",
          border: "1px solid var(--color-border)",
          borderRadius: "var(--border-radius)",
          padding: "var(--space-6)",
        }}
      >
        <div
          className="flex justify-between items-center"
          style={{ marginBottom: "var(--space-6)" }}
        >
          <h2
            className="font-bold"
            style={{
              color: "var(--color-primary)",
              fontFamily: "var(--font-body)",
              fontSize: "1.25rem",
              lineHeight: "var(--line-height-tight)",
            }}
          >
            Settings
          </h2>
          <button
            type="button"
            className="oracle-btn oracle-btn-secondary"
            onClick={closeSettings}
          >
            ✕ Close
          </button>
        </div>
        <div style={{ marginBottom: "var(--space-6)" }}>
          <LlmProviderSelector />
        </div>
        <div
          style={{
            borderTop: "1px solid var(--color-border)",
            paddingTop: "var(--space-6)",
          }}
        >
          <ThemeSelector />
        </div>
      </div>
    </div>
  );
}
