import { useState } from "react";
import { useCampaignStore } from "../../stores/campaignStore";
import { useUiStore } from "../../stores/uiStore";
import { suggestedThemeForSystem } from "../../themes";
import { useSettingsStore } from "../../stores/settingsStore";

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

export function NewCampaignModal() {
  const [name, setName] = useState("");
  const [systemId, setSystemId] = useState("cairn");
  const { createCampaign } = useCampaignStore();
  const { closeNewCampaignModal } = useUiStore();
  const { updateSettings } = useSettingsStore();

  async function handleCreate() {
    if (!name.trim()) return;
    await createCampaign(name.trim(), systemId);
    const suggested = suggestedThemeForSystem(systemId);
    await updateSettings({ theme: suggested });
    closeNewCampaignModal();
    setName("");
  }

  return (
    <div className="fixed inset-0 flex items-center justify-center z-50" style={{ backgroundColor: "rgba(0,0,0,0.7)" }}>
      <div
        className="w-96 p-6 rounded-lg"
        style={{
          backgroundColor: "var(--color-surface)",
          border: "1px solid var(--color-border)",
          borderRadius: "var(--border-radius)",
        }}
      >
        <h2 className="text-lg font-bold mb-4" style={{ color: "var(--color-primary)", fontFamily: "var(--font-body)" }}>
          New Campaign
        </h2>
        <label className="block text-sm mb-1" style={{ color: "var(--color-text-muted)" }}>Campaign Name</label>
        <input
          className="w-full p-2 mb-4 text-sm focus:outline-none"
          style={{
            backgroundColor: "var(--color-bg)",
            border: "1px solid var(--color-border)",
            color: "var(--color-text)",
            borderRadius: "var(--border-radius)",
          }}
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="My Adventure"
          autoFocus
        />
        <label className="block text-sm mb-1" style={{ color: "var(--color-text-muted)" }}>RPG System</label>
        <select
          className="w-full p-2 mb-6 text-sm focus:outline-none"
          style={{
            backgroundColor: "var(--color-bg)",
            border: "1px solid var(--color-border)",
            color: "var(--color-text)",
            borderRadius: "var(--border-radius)",
          }}
          value={systemId}
          onChange={(e) => setSystemId(e.target.value)}
        >
          {RPG_SYSTEMS.map((s) => (
            <option key={s.id} value={s.id}>{s.name}</option>
          ))}
        </select>
        <div className="flex gap-3 justify-end">
          <button
            className="px-4 py-2 text-sm"
            style={{ color: "var(--color-text-muted)" }}
            onClick={closeNewCampaignModal}
          >
            Cancel
          </button>
          <button
            className="px-4 py-2 text-sm font-semibold"
            style={{
              backgroundColor: "var(--color-primary)",
              color: "var(--color-bg)",
              borderRadius: "var(--border-radius)",
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
