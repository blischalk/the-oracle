import { useUiStore } from "../../stores/uiStore";
import { LlmProviderSelector } from "./LlmProviderSelector";
import { ThemeSelector } from "./ThemeSelector";

export function SettingsPanel() {
  const { closeSettings } = useUiStore();

  return (
    <div className="fixed inset-0 flex items-center justify-center z-50" style={{ backgroundColor: "rgba(0,0,0,0.7)" }}>
      <div
        className="w-[480px] max-h-[80vh] overflow-y-auto p-6 rounded-lg"
        style={{
          backgroundColor: "var(--color-surface)",
          border: "1px solid var(--color-border)",
          borderRadius: "var(--border-radius)",
        }}
      >
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-lg font-bold" style={{ color: "var(--color-primary)", fontFamily: "var(--font-body)" }}>
            Settings
          </h2>
          <button
            className="text-sm"
            style={{ color: "var(--color-text-muted)" }}
            onClick={closeSettings}
          >
            ✕ Close
          </button>
        </div>
        <div className="mb-6">
          <LlmProviderSelector />
        </div>
        <div style={{ borderTop: "1px solid var(--color-border)", paddingTop: "1.5rem" }}>
          <ThemeSelector />
        </div>
      </div>
    </div>
  );
}
