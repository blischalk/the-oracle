import { useCampaignStore } from "../../stores/campaignStore";
import { useUiStore } from "../../stores/uiStore";
import { CampaignCard } from "./CampaignCard";

export function CampaignSidebar() {
  const { campaigns, activeCampaignId, selectCampaign, archiveCampaign } = useCampaignStore();
  const { openNewCampaignModal, openSettings } = useUiStore();

  return (
    <aside
      className="w-64 flex flex-col h-full"
      style={{
        backgroundColor: "var(--color-surface)",
        borderRight: "1px solid var(--color-border)",
      }}
    >
      <div className="p-4" style={{ borderBottom: "1px solid var(--color-border)" }}>
        <h1 className="text-xl font-bold" style={{ color: "var(--color-primary)", fontFamily: "var(--font-body)" }}>
          The Oracle
        </h1>
      </div>
      <div className="flex-1 overflow-y-auto p-3">
        {campaigns.map((c) => (
          <CampaignCard
            key={c.id}
            campaign={c}
            isActive={c.id === activeCampaignId}
            onSelect={() => selectCampaign(c.id)}
            onArchive={() => archiveCampaign(c.id)}
          />
        ))}
        {campaigns.length === 0 && (
          <p className="text-xs text-center mt-8" style={{ color: "var(--color-text-muted)" }}>
            No campaigns yet
          </p>
        )}
      </div>
      <div className="p-3 flex flex-col gap-2" style={{ borderTop: "1px solid var(--color-border)" }}>
        <button
          className="w-full py-2 text-sm font-semibold"
          style={{
            backgroundColor: "var(--color-primary)",
            color: "var(--color-bg)",
            borderRadius: "var(--border-radius)",
          }}
          onClick={openNewCampaignModal}
        >
          + New Campaign
        </button>
        <button
          className="w-full py-2 text-sm"
          style={{
            border: "1px solid var(--color-border)",
            color: "var(--color-text-muted)",
            borderRadius: "var(--border-radius)",
          }}
          onClick={openSettings}
        >
          Settings
        </button>
      </div>
    </aside>
  );
}
