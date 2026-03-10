import type { SidebarTab } from "../../stores/uiStore";

interface TabDefinition {
  id: SidebarTab;
  label: string;
  icon: React.ReactNode;
}

const TABS: TabDefinition[] = [
  {
    id: "campaigns",
    label: "Campaigns",
    icon: (
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z" />
        <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z" />
      </svg>
    ),
  },
  {
    id: "character",
    label: "Character",
    icon: (
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <circle cx="12" cy="8" r="4" />
        <path d="M4 20c0-4 3.6-7 8-7s8 3 8 7" />
      </svg>
    ),
  },
  {
    id: "inventory",
    label: "Inventory",
    icon: (
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M4 20V10a2 2 0 0 1 2-2h2V6a4 4 0 0 1 8 0v2h2a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2z" />
        <path d="M10 6v2M14 6v2" />
      </svg>
    ),
  },
  {
    id: "skills",
    label: "Skills",
    icon: (
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M12 2L9.1 8.6 2 9.5l5 4.9-1.2 6.9L12 18l6.2 3.3L17 14.4l5-4.9-7.1-.9z" />
      </svg>
    ),
  },
  {
    id: "journal",
    label: "Journal",
    icon: (
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" />
        <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" />
      </svg>
    ),
  },
  {
    id: "quests",
    label: "Quests",
    icon: (
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M4 15s1-1 4-1 5 2 8 2 4-1 4-1V3s-1 1-4 1-5-2-8-2-4 1-4 1z" />
        <line x1="4" y1="22" x2="4" y2="15" />
      </svg>
    ),
  },
];

interface SidebarTabBarProps {
  isSidebarOpen: boolean;
  onToggleSidebar: () => void;
  isFullscreen: boolean;
  onToggleFullscreen: () => void;
  activeTab: SidebarTab;
  isCampaignActive: boolean;
  onTabSelect: (tab: SidebarTab) => void;
}

function IconButton({
  label,
  onClick,
  children,
}: {
  label: string;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      title={label}
      onClick={onClick}
      aria-label={label}
      style={{
        width: "40px",
        height: "40px",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        borderRadius: "var(--border-radius)",
        border: "none",
        cursor: "pointer",
        backgroundColor: "transparent",
        color: "var(--color-text-muted)",
        transition: "background-color 0.15s, color 0.15s",
      }}
    >
      {children}
    </button>
  );
}

export function SidebarTabBar({
  isSidebarOpen,
  onToggleSidebar,
  isFullscreen,
  onToggleFullscreen,
  activeTab,
  isCampaignActive,
  onTabSelect,
}: SidebarTabBarProps) {
  return (
    <div
      className="flex flex-col items-center"
      style={{
        width: "48px",
        borderRight: "1px solid var(--color-border)",
        backgroundColor: "var(--color-surface)",
        paddingTop: "var(--space-2)",
        paddingBottom: "var(--space-2)",
        gap: "4px",
        flexShrink: 0,
      }}
    >
      <IconButton
        label={isSidebarOpen ? "Collapse sidebar" : "Expand sidebar"}
        onClick={onToggleSidebar}
      >
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <line x1="3" y1="6" x2="21" y2="6" />
          <line x1="3" y1="12" x2="21" y2="12" />
          <line x1="3" y1="18" x2="21" y2="18" />
        </svg>
      </IconButton>

      <IconButton
        label={isFullscreen ? "Exit fullscreen (Esc)" : "Enter fullscreen"}
        onClick={onToggleFullscreen}
      >
        {isFullscreen ? (
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M8 3v3a2 2 0 0 1-2 2H3" />
            <path d="M21 8h-3a2 2 0 0 1-2-2V3" />
            <path d="M3 16h3a2 2 0 0 1 2 2v3" />
            <path d="M16 21v-3a2 2 0 0 1 2-2h3" />
          </svg>
        ) : (
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M3 7V3h4" />
            <path d="M21 7V3h-4" />
            <path d="M3 17v4h4" />
            <path d="M21 17v4h-4" />
          </svg>
        )}
      </IconButton>

      <div style={{ width: "32px", height: "1px", backgroundColor: "var(--color-border)", margin: "4px 0" }} />

      {TABS.map((tab) => {
        const isCampaignRequired = tab.id !== "campaigns";
        const isDisabled = isCampaignRequired && !isCampaignActive;
        const isActive = activeTab === tab.id;

        return (
          <button
            key={tab.id}
            type="button"
            title={tab.label}
            disabled={isDisabled}
            onClick={() => onTabSelect(tab.id)}
            style={{
              width: "40px",
              height: "40px",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              borderRadius: "var(--border-radius)",
              border: "none",
              cursor: isDisabled ? "not-allowed" : "pointer",
              backgroundColor: isActive ? "var(--color-primary)" : "transparent",
              color: isActive
                ? "var(--color-bg)"
                : isDisabled
                ? "var(--color-border)"
                : "var(--color-text-muted)",
              transition: "background-color 0.15s, color 0.15s",
            }}
            aria-label={tab.label}
            aria-pressed={isActive}
          >
            {tab.icon}
          </button>
        );
      })}
    </div>
  );
}
