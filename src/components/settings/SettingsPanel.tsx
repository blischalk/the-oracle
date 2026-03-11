import { useUiStore } from "../../stores/uiStore";
import { LlmProviderSelector } from "./LlmProviderSelector";
import { ThemeSelector } from "./ThemeSelector";
import { NarrationSettings } from "./NarrationSettings";
import { openUserSystemsFolder } from "../../services/campaignService";

const sectionDivider: React.CSSProperties = {
  borderTop: "1px solid var(--color-border)",
  paddingTop: "var(--space-6)",
  marginTop: "var(--space-6)",
};

export function SettingsPanel() {
  const { closeSettings } = useUiStore();

  async function handleOpenSystemsFolder() {
    try {
      await openUserSystemsFolder();
    } catch {
      // Opener errors are non-fatal; the user can navigate manually
    }
  }

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

        <LlmProviderSelector />

        <div style={sectionDivider}>
          <ThemeSelector />
        </div>

        <div style={sectionDivider}>
          <NarrationSettings />
        </div>

        <div style={sectionDivider}>
          <h3
            className="oracle-label"
            style={{ marginBottom: "var(--space-3)", display: "block" }}
          >
            Custom RPG Systems
          </h3>
          <p
            style={{
              color: "var(--color-text-muted)",
              fontFamily: "var(--font-body)",
              fontSize: "0.875rem",
              lineHeight: "1.6",
              marginBottom: "var(--space-4)",
            }}
          >
            Add your own RPG systems by dropping <code>.yaml</code> files into
            the user systems folder. Custom systems appear alongside the built-in
            ones when creating a new campaign.
          </p>
          <button
            type="button"
            className="oracle-btn oracle-btn-secondary"
            onClick={handleOpenSystemsFolder}
          >
            Open Systems Folder
          </button>
        </div>
      </div>
    </div>
  );
}
