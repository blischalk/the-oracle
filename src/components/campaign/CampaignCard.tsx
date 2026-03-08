import { Campaign } from "../../domain/campaign";

const RPG_NAMES: Record<string, string> = {
  "old-school-essentials": "Old School Essentials",
  "troika": "Troika!",
  "mork-borg": "Mörk Borg",
  "cairn": "Cairn",
  "into-the-odd": "Into the Odd",
  "electric-bastionland": "Electric Bastionland",
  "ultraviolet-grasslands": "Ultraviolet Grasslands",
  "runecairn": "Runecairn",
  "between-the-skies": "Between the Skies",
};

interface Props {
  campaign: Campaign;
  isActive: boolean;
  onSelect: () => void;
  onArchive: () => void;
}

export function CampaignCard({ campaign, isActive, onSelect, onArchive }: Props) {
  return (
    <div
      className="p-3 mb-2 cursor-pointer transition-all"
      style={{
        backgroundColor: isActive ? "var(--color-border)" : "var(--color-surface)",
        border: `1px solid ${isActive ? "var(--color-primary)" : "var(--color-border)"}`,
        borderRadius: "var(--border-radius)",
      }}
      onClick={onSelect}
    >
      <p className="font-semibold text-sm truncate" style={{ color: "var(--color-text)" }}>
        {campaign.name}
      </p>
      <p className="text-xs mt-1" style={{ color: "var(--color-text-muted)" }}>
        {RPG_NAMES[campaign.rpg_system_id] ?? campaign.rpg_system_id}
      </p>
      <button
        className="text-xs mt-2 opacity-50 hover:opacity-100"
        style={{ color: "var(--color-text-muted)" }}
        onClick={(e) => {
          e.stopPropagation();
          onArchive();
        }}
      >
        Archive
      </button>
    </div>
  );
}
