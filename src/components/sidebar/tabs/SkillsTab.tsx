import { useCampaignStore } from "../../../stores/campaignStore";
import { getFieldTab } from "../fieldTabHelper";

export function SkillsTab() {
  const { activeCampaignId, activeRpgSystem, campaignState } = useCampaignStore();

  if (!activeCampaignId || !activeRpgSystem) {
    return (
      <div style={{ padding: "var(--space-4)", color: "var(--color-text-muted)", fontSize: "0.875rem" }}>
        Select a campaign to view skills.
      </div>
    );
  }

  const characterData = (campaignState?.character_data ?? {}) as Record<string, unknown>;
  const skillsFields = activeRpgSystem.character_fields.filter(
    (field) => getFieldTab(field.name) === "skills"
  );

  if (skillsFields.length === 0) {
    return (
      <div style={{ padding: "var(--space-4)", color: "var(--color-text-muted)", fontSize: "0.875rem" }}>
        No skills defined for this system.
      </div>
    );
  }

  return (
    <div className="overflow-y-auto" style={{ padding: "var(--space-3)" }}>
      <div className="flex flex-col gap-3">
        {skillsFields.map((field) => {
          const value = characterData[field.name];
          const display =
            value !== undefined && value !== null && value !== ""
              ? String(value)
              : "—";
          return (
            <div key={field.name}>
              <div
                style={{
                  fontSize: "0.75rem",
                  color: "var(--color-text-muted)",
                  marginBottom: "2px",
                  textTransform: "uppercase",
                  letterSpacing: "0.05em",
                }}
              >
                {field.label}
              </div>
              <div
                style={{
                  fontSize: "0.9375rem",
                  color: "var(--color-text)",
                  fontWeight: 500,
                  wordBreak: "break-word",
                }}
              >
                {display}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
