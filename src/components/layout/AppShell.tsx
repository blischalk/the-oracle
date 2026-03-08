import { useEffect } from "react";
import { CampaignSidebar } from "../campaign/CampaignSidebar";
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
  const { isSidebarOpen, isSettingsOpen, isNewCampaignModalOpen, toggleSidebar, openSettings, closeSettings } = useUiStore();
  const { updateSettings, settings } = useSettings();

  // Initialize theme and campaigns
  useCampaigns();
  useTheme();

  // Keyboard shortcuts
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape" && settings.is_fullscreen) {
        updateSettings({ is_fullscreen: false });
      }
      if ((e.metaKey || e.ctrlKey) && e.key === ",") {
        e.preventDefault();
        if (isSettingsOpen) { closeSettings(); } else { openSettings(); }
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [settings.is_fullscreen, isSettingsOpen]);

  return (
    <div className="flex flex-col h-screen">
      <div className="flex flex-1 overflow-hidden">
        {isSidebarOpen && <CampaignSidebar />}
        <div className="flex flex-col flex-1 overflow-hidden" style={{ backgroundColor: "var(--color-bg)" }}>
          <div
            className="flex items-center gap-2 px-4 py-2"
            style={{ borderBottom: "1px solid var(--color-border)", backgroundColor: "var(--color-surface)" }}
          >
            <button
              className="text-sm"
              style={{ color: "var(--color-text-muted)" }}
              onClick={toggleSidebar}
              title="Toggle sidebar"
            >
              ☰
            </button>
            <div className="flex-1" />
            <FullscreenToggle />
          </div>
          <ChatWindow />
        </div>
      </div>
      <StatusBar />
      {isSettingsOpen && <SettingsPanel />}
      {isNewCampaignModalOpen && <NewCampaignModal />}
    </div>
  );
}
