import { useSettingsStore } from "../../stores/settingsStore";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function FullscreenToggle() {
  const { settings, updateSettings } = useSettingsStore();

  async function toggle() {
    const win = getCurrentWindow();
    const next = !settings.is_fullscreen;
    await win.setFullscreen(next);
    await updateSettings({ is_fullscreen: next });
  }

  return (
    <button
      className="text-xs px-2 py-1"
      style={{ color: "var(--color-text-muted)" }}
      onClick={toggle}
      title={settings.is_fullscreen ? "Exit Fullscreen (Esc)" : "Enter Fullscreen"}
    >
      {settings.is_fullscreen ? "⤡" : "⤢"}
    </button>
  );
}
