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
        width: isSidebarOpen ? "256px" : "48px",
        backgroundColor: "var(--color-surface)",
        borderRight: "1px solid var(--color-border)",
        flexShrink: 0,
        transition: "width 0.2s ease",
        overflow: "hidden",
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
    </aside>
  );
}
