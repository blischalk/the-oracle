import { useState, useEffect, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCampaignStore } from "../../stores/campaignStore";
import { useUiStore } from "../../stores/uiStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { SidebarTabBar } from "./SidebarTabBar";
import { CampaignsTab } from "./tabs/CampaignsTab";
import { CharacterTab } from "./tabs/CharacterTab";
import { InventoryTab } from "./tabs/InventoryTab";
import { SkillsTab } from "./tabs/SkillsTab";
import { JournalTab } from "./tabs/JournalTab";
import { QuestsTab } from "./tabs/QuestsTab";

const MIN_SIDEBAR_WIDTH = 180;
const MAX_SIDEBAR_WIDTH = 600;
const DEFAULT_SIDEBAR_WIDTH = 320;

function TabContent({ activeTab }: { activeTab: string }) {
  switch (activeTab) {
    case "campaigns":
      return <CampaignsTab />;
    case "character":
      return <CharacterTab />;
    case "inventory":
      return <InventoryTab />;
    case "skills":
      return <SkillsTab />;
    case "journal":
      return <JournalTab />;
    case "quests":
      return <QuestsTab />;
    default:
      return <CampaignsTab />;
  }
}

export function Sidebar() {
  const { activeCampaignId } = useCampaignStore();
  const { isSidebarOpen, toggleSidebar, activeSidebarTab, setActiveSidebarTab } = useUiStore();
  const { settings, updateSettings } = useSettingsStore();
  const [sidebarWidth, setSidebarWidth] = useState(DEFAULT_SIDEBAR_WIDTH);
  const [isResizing, setIsResizing] = useState(false);
  const dragState = useRef({ startX: 0, startWidth: 0 });

  useEffect(() => {
    if (!isResizing) return;

    function handleMouseMove(e: MouseEvent) {
      const delta = e.clientX - dragState.current.startX;
      const newWidth = Math.max(MIN_SIDEBAR_WIDTH, Math.min(MAX_SIDEBAR_WIDTH, dragState.current.startWidth + delta));
      setSidebarWidth(newWidth);
    }

    function handleMouseUp() {
      setIsResizing(false);
    }

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };
  }, [isResizing]);

  function handleResizeStart(e: React.MouseEvent) {
    dragState.current = { startX: e.clientX, startWidth: sidebarWidth };
    setIsResizing(true);
    e.preventDefault();
  }

  async function handleToggleFullscreen() {
    try {
      const next = !settings.is_fullscreen;
      await getCurrentWindow().setFullscreen(next);
      await updateSettings({ is_fullscreen: next });
    } catch (err) {
      console.error("Failed to toggle fullscreen:", err);
    }
  }

  function handleTabSelect(tab: typeof activeSidebarTab) {
    if (tab !== "campaigns" && !activeCampaignId) {
      setActiveSidebarTab("campaigns");
      return;
    }
    setActiveSidebarTab(tab);
  }

  return (
    <aside
      className="flex h-full"
      style={{
        position: "relative",
        width: isSidebarOpen ? `${sidebarWidth}px` : "48px",
        backgroundColor: "var(--color-surface)",
        borderRight: "1px solid var(--color-border)",
        flexShrink: 0,
        transition: isResizing ? "none" : "width 0.2s ease",
      }}
    >
      <SidebarTabBar
        isSidebarOpen={isSidebarOpen}
        onToggleSidebar={toggleSidebar}
        isFullscreen={settings.is_fullscreen}
        onToggleFullscreen={handleToggleFullscreen}
        activeTab={activeSidebarTab}
        isCampaignActive={!!activeCampaignId}
        onTabSelect={handleTabSelect}
      />
      {isSidebarOpen && (
        <div
          className="flex flex-col flex-1 overflow-hidden"
          style={{ borderLeft: "1px solid var(--color-border)" }}
        >
          <div
            style={{
              padding: "var(--space-3) var(--space-4)",
              borderBottom: "1px solid var(--color-border)",
            }}
          >
            <h1
              className="font-bold"
              style={{
                color: "var(--color-primary)",
                fontFamily: "var(--font-body)",
                fontSize: "1.125rem",
                lineHeight: "var(--line-height-tight)",
              }}
            >
              The Oracle
            </h1>
          </div>
          <div className="flex-1 overflow-hidden flex flex-col">
            <TabContent activeTab={activeSidebarTab} />
          </div>
        </div>
      )}
      {isSidebarOpen && (
        <div
          style={{
            position: "absolute",
            right: 0,
            top: 0,
            bottom: 0,
            width: "6px",
            cursor: "ew-resize",
            zIndex: 10,
          }}
          onMouseDown={handleResizeStart}
        />
      )}
    </aside>
  );
}
