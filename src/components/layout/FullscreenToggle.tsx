import { useSettingsStore } from "../../stores/settingsStore";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function FullscreenToggle() {
  const { settings, updateSettings } = useSettingsStore();

  async function toggle() {
    try {
      const win = getCurrentWindow();
      const next = !settings.is_fullscreen;
      await win.setFullscreen(next);
      await updateSettings({ is_fullscreen: next });
    } catch (err) {
      console.error("Failed to toggle fullscreen:", err);
    }
  }

  return (
    <button
      type="button"
      style={{
        color: "var(--color-text-muted)",
        padding: "var(--space-2) var(--space-3)",
        fontSize: "0.8125rem",
        background: "none",
        border: "none",
        cursor: "pointer",
        borderRadius: "var(--border-radius)",
      }}
      onClick={toggle}
      title={settings.is_fullscreen ? "Exit Fullscreen (Esc)" : "Enter Fullscreen"}
    >
      {settings.is_fullscreen ? "⤡" : "⤢"}
    </button>
  );
}
