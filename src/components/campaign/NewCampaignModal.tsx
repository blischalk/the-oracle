import { useState } from "react";
import { useCampaignStore } from "../../stores/campaignStore";
import { useUiStore } from "../../stores/uiStore";
import { suggestedThemeForSystem } from "../../themes";
import { useSettingsStore } from "../../stores/settingsStore";
import { ThemedSelect } from "../ui/ThemedSelect";

const RPG_SYSTEMS = [
  { id: "cairn", name: "Cairn" },
  { id: "old-school-essentials", name: "Old School Essentials" },
  { id: "troika", name: "Troika!" },
  { id: "mork-borg", name: "Mörk Borg" },
  { id: "into-the-odd", name: "Into the Odd" },
  { id: "electric-bastionland", name: "Electric Bastionland" },
  { id: "ultraviolet-grasslands", name: "Ultraviolet Grasslands" },
  { id: "runecairn", name: "Runecairn" },
  { id: "between-the-skies", name: "Between the Skies" },
];

const PLACEHOLDER_CAMPAIGN_NAME = "New Adventure";

export function NewCampaignModal() {
  const [systemId, setSystemId] = useState("cairn");
  const { createCampaign } = useCampaignStore();
  const { closeNewCampaignModal } = useUiStore();
  const { updateSettings } = useSettingsStore();

  async function handleCreate() {
    await createCampaign(PLACEHOLDER_CAMPAIGN_NAME, systemId);
    const suggested = suggestedThemeForSystem(systemId);
    await updateSettings({ theme: suggested });
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
          <ThemedSelect
            id="rpg-system"
            options={RPG_SYSTEMS.map((s) => ({ value: s.id, label: s.name }))}
            value={systemId}
            onChange={setSystemId}
          />
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
          >
            Create
          </button>
        </div>
      </div>
    </div>
  );
}
