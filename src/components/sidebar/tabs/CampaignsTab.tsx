import { useCampaignStore } from "../../../stores/campaignStore";
import { useUiStore } from "../../../stores/uiStore";
import { exportCampaignToMarkdown } from "../../../services/campaignService";
import { CampaignCard } from "../../campaign/CampaignCard";

export function CampaignsTab() {
  const {
    campaigns,
    activeCampaignId,
    activeCampaign,
    messages,
    selectCampaign,
    updateCampaignName,
    deleteCampaign,
    error,
    clearError,
  } = useCampaignStore();
  const { openNewCampaignModal, openSettings } = useUiStore();

  function handleExport() {
    if (!activeCampaign) return;
    const markdown = exportCampaignToMarkdown(activeCampaign, messages);
    const blob = new Blob([markdown], { type: "text/markdown" });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = `${activeCampaign.name.replace(/[^a-z0-9]/gi, "-").toLowerCase()}.md`;
    anchor.click();
    URL.revokeObjectURL(url);
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex-1 overflow-y-auto" style={{ padding: "var(--space-3)" }}>
        {error && (
          <div
            role="alert"
            style={{
              padding: "var(--space-2)",
              marginBottom: "var(--space-2)",
              backgroundColor: "var(--color-error-bg, #fef2f2)",
              color: "var(--color-error, #b91c1c)",
              fontSize: "0.8125rem",
              borderRadius: "var(--border-radius)",
            }}
          >
            <span>{error}</span>
            <button
              type="button"
              onClick={clearError}
              style={{ marginLeft: "var(--space-2)", textDecoration: "underline" }}
              aria-label="Dismiss error"
            >
              Dismiss
            </button>
          </div>
        )}
        {campaigns.map((campaign) => (
          <CampaignCard
            key={campaign.id}
            campaign={campaign}
            isActive={campaign.id === activeCampaignId}
            onSelect={() => selectCampaign(campaign.id)}
            onRename={(name) => updateCampaignName(campaign.id, name)}
            onDelete={() => deleteCampaign(campaign.id)}
          />
        ))}
        {campaigns.length === 0 && (
          <p className="text-xs text-center mt-8" style={{ color: "var(--color-text-muted)" }}>
            No campaigns yet
          </p>
        )}
      </div>
      <div
        className="flex flex-col gap-2"
        style={{
          padding: "var(--space-4)",
          borderTop: "1px solid var(--color-border)",
        }}
      >
        <button
          type="button"
          className="oracle-btn w-full"
          style={{
            backgroundColor: "var(--color-primary)",
            color: "var(--color-bg)",
          }}
          onClick={openNewCampaignModal}
        >
          + New Campaign
        </button>
        <button
          type="button"
          className="oracle-btn oracle-btn-secondary w-full"
          onClick={openSettings}
        >
          Settings
        </button>
        {activeCampaignId && activeCampaign && (
          <button
            type="button"
            className="w-full py-2 text-sm"
            style={{
              border: "1px solid var(--color-border)",
              color: "var(--color-text-muted)",
              borderRadius: "var(--border-radius)",
            }}
            onClick={handleExport}
          >
            Export Campaign
          </button>
        )}
      </div>
    </div>
  );
}
