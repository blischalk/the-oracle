import { useState, useEffect } from "react";
import { useCampaignStore } from "../../stores/campaignStore";
import { useUiStore } from "../../stores/uiStore";
import { ThemedSelect } from "../ui/ThemedSelect";
import * as campaignService from "../../services/campaignService";
import type { RpgSystem } from "../../domain/rpgSystem";

const PLACEHOLDER_CAMPAIGN_NAME = "New Adventure";

export function NewCampaignModal() {
  const [systems, setSystems] = useState<RpgSystem[]>([]);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [systemId, setSystemId] = useState("");
  const { createCampaign } = useCampaignStore();
  const { closeNewCampaignModal } = useUiStore();

  useEffect(() => {
    campaignService.listRpgSystems().then((loaded) => {
      setSystems(loaded);
      if (loaded.length > 0 && !systemId) {
        setSystemId(loaded[0].id);
      }
    }).catch((err: unknown) => {
      setLoadError(String(err));
    });
  }, []);

  async function handleCreate() {
    if (!systemId) return;
    await createCampaign(PLACEHOLDER_CAMPAIGN_NAME, systemId);
    closeNewCampaignModal();
  }

  return (
    <div className="fixed inset-0 flex items-center justify-center z-50" style={{ backgroundColor: "rgba(0,0,0,0.7)" }}>
      <div
        className="w-[28rem] rounded-lg"
        style={{
          backgroundColor: "var(--color-surface)",
          border: "1px solid var(--color-border)",
          borderRadius: "var(--border-radius)",
          padding: "var(--space-6)",
        }}
      >
        <h2
          className="font-bold"
          style={{
            color: "var(--color-primary)",
            fontFamily: "var(--font-body)",
            fontSize: "1.25rem",
            lineHeight: "var(--line-height-tight)",
            marginBottom: "var(--space-5)",
          }}
        >
          New Campaign
        </h2>
        <div className="oracle-form-group">
          <label className="oracle-label" htmlFor="rpg-system">
            RPG System
          </label>
          {loadError ? (
            <p style={{ color: "red", fontFamily: "var(--font-body)", fontSize: "0.875rem" }}>
              Error loading systems: {loadError}
            </p>
          ) : systems.length === 0 ? (
            <p style={{ color: "var(--color-text-muted)", fontFamily: "var(--font-body)", fontSize: "0.875rem" }}>
              Loading systems…
            </p>
          ) : (
            <ThemedSelect
              id="rpg-system"
              options={systems.map((s) => ({ value: s.id, label: s.name }))}
              value={systemId}
              onChange={setSystemId}
            />
          )}
        </div>
        <div
          className="flex gap-3 justify-end"
          style={{ marginTop: "var(--space-6)", paddingTop: "var(--space-4)" }}
        >
          <button
            type="button"
            className="oracle-btn oracle-btn-secondary"
            onClick={closeNewCampaignModal}
          >
            Cancel
          </button>
          <button
            type="button"
            className="oracle-btn"
            style={{
              backgroundColor: "var(--color-primary)",
              color: "var(--color-bg)",
            }}
            onClick={handleCreate}
            disabled={!systemId}
          >
            Create
          </button>
        </div>
      </div>
    </div>
  );
}
