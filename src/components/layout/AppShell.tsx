import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { CampaignSidebar } from "../campaign/CampaignSidebar";
import { CharacterProfile } from "../campaign/CharacterProfile";
import { ChatWindow } from "../chat/ChatWindow";
import { SettingsPanel } from "../settings/SettingsPanel";
import { NewCampaignModal } from "../campaign/NewCampaignModal";
import { StatusBar } from "./StatusBar";
import { FullscreenToggle } from "./FullscreenToggle";
import { useUiStore } from "../../stores/uiStore";
import { useCampaigns } from "../../hooks/useCampaign";
import { useSettings } from "../../hooks/useSettings";
import { useTheme } from "../../hooks/useTheme";

export function AppShell() {
  const { isSidebarOpen, isSettingsOpen, isNewCampaignModalOpen, toggleSidebar, openSettings, closeSettings, openNewCampaignModal } = useUiStore();
  const { updateSettings, settings } = useSettings();

  // Initialize theme and campaigns
  useCampaigns();
  useTheme();

  // Apply stored fullscreen preference so window and store stay in sync (fixes single-click toggle)
  useEffect(() => {
    if (!settings.is_fullscreen) return;
    getCurrentWindow()
      .setFullscreen(true)
      .catch(() => {});
  }, [settings.is_fullscreen]);

  // Keyboard shortcuts
  useEffect(() => {
    async function handleKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape" && settings.is_fullscreen) {
        try {
          await getCurrentWindow().setFullscreen(false);
          updateSettings({ is_fullscreen: false });
        } catch {
          updateSettings({ is_fullscreen: false });
        }
      }
      if ((e.metaKey || e.ctrlKey) && e.key === "n") {
        e.preventDefault();
        openNewCampaignModal();
      }
      if ((e.metaKey || e.ctrlKey) && e.key === ",") {
        e.preventDefault();
        if (isSettingsOpen) { closeSettings(); } else { openSettings(); }
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [settings.is_fullscreen, isSettingsOpen, openNewCampaignModal, closeSettings, openSettings, updateSettings]);

  return (
    <div className="flex flex-col h-screen">
      <div className="flex flex-1 overflow-hidden">
        {isSidebarOpen && <CampaignSidebar />}
        <div className="flex flex-col flex-1 overflow-hidden min-h-0" style={{ backgroundColor: "var(--color-bg)" }}>
          <div
            className="flex items-center gap-2 flex-shrink-0"
            style={{
              padding: "var(--space-2) var(--space-4)",
              borderBottom: "1px solid var(--color-border)",
              backgroundColor: "var(--color-surface)",
            }}
          >
            <button
              type="button"
              style={{
                padding: "var(--space-2) var(--space-3)",
                fontSize: "0.9375rem",
                color: "var(--color-text-muted)",
                background: "none",
                border: "none",
                cursor: "pointer",
                borderRadius: "var(--border-radius)",
              }}
              onClick={toggleSidebar}
              title="Toggle sidebar"
            >
              ☰
            </button>
            <div className="flex-1" />
            <FullscreenToggle />
          </div>
          <CharacterProfile />
          <div className="flex-1 min-h-0 flex flex-col overflow-hidden">
            <ChatWindow />
          </div>
        </div>
      </div>
      <StatusBar />
      {isSettingsOpen && <SettingsPanel />}
      {isNewCampaignModalOpen && <NewCampaignModal />}
    </div>
  );
}
