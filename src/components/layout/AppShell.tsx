import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Sidebar } from "../sidebar/Sidebar";
import { CharacterProfile } from "../campaign/CharacterProfile";
import { ChatWindow } from "../chat/ChatWindow";
import { SettingsPanel } from "../settings/SettingsPanel";
import { NewCampaignModal } from "../campaign/NewCampaignModal";
import { StatusBar } from "./StatusBar";
import { useUiStore } from "../../stores/uiStore";
import { useCampaigns } from "../../hooks/useCampaign";
import { useSettings } from "../../hooks/useSettings";
import { useTheme } from "../../hooks/useTheme";

export function AppShell() {
  const { isSettingsOpen, isNewCampaignModalOpen, openSettings, closeSettings, openNewCampaignModal } = useUiStore();
  const { updateSettings, settings } = useSettings();

  useCampaigns();
  useTheme();

  // Apply stored fullscreen preference on mount so window and store stay in sync
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
        <Sidebar />
        <div className="flex flex-col flex-1 overflow-hidden min-h-0" style={{ backgroundColor: "var(--color-bg)" }}>
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
