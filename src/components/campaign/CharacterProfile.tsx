import { useCampaignStore } from "../../stores/campaignStore";


function getCharacterName(characterData: Record<string, unknown>): string {
  const name =
    characterData.character_name ??
    characterData.name ??
    characterData.CharacterName;
  if (typeof name === "string" && name.trim()) return name.trim();
  return "—";
}

export function CharacterProfile() {
  const { activeCampaignId, activeCampaign, campaignState, activeRpgSystem } =
    useCampaignStore();

  if (!activeCampaignId || !activeCampaign || !activeRpgSystem) {
    return null;
  }

  const characterData = (campaignState?.character_data ?? {}) as Record<
    string,
    unknown
  >;
  const characterName = getCharacterName(characterData);

  const statFields = activeRpgSystem.character_fields.filter(
    (field) =>
      typeof field.field_type === "object" &&
      field.field_type !== null &&
      "type" in field.field_type &&
      (field.field_type as { type: string }).type === "Number"
  );

  return (
    <div
      className="flex items-center gap-4 flex-wrap"
      style={{
        padding: "var(--space-2) var(--space-4)",
        borderBottom: "1px solid var(--color-border)",
        backgroundColor: "var(--color-surface)",
        fontFamily: "var(--font-body)",
        fontSize: "0.875rem",
      }}
    >
      <div className="flex items-center gap-2">
        <span
          style={{
            color: "var(--color-text-muted)",
            fontWeight: 500,
          }}
        >
          Character
        </span>
        <span
          style={{
            color: "var(--color-text)",
            fontWeight: 600,
          }}
        >
          {characterName}
        </span>
      </div>
      <span
        style={{
          color: "var(--color-border)",
          width: "1px",
          alignSelf: "stretch",
        }}
        aria-hidden
      />
      <div className="flex items-center gap-2 flex-wrap">
        <span
          style={{
            color: "var(--color-text-muted)",
            fontWeight: 500,
          }}
        >
          {activeRpgSystem.name}
        </span>
        {statFields.length > 0 && (
          <>
            <span
              style={{
                color: "var(--color-border)",
                width: "1px",
                alignSelf: "stretch",
              }}
              aria-hidden
            />
            <div className="flex items-center gap-3 flex-wrap">
              {statFields.map((field) => {
                const value = characterData[field.name];
                const display =
                  value !== undefined && value !== null && value !== ""
                    ? String(value)
                    : "—";
                return (
                  <span
                    key={field.name}
                    className="flex items-center gap-1"
                    style={{ color: "var(--color-text)" }}
                  >
                    <span
                      style={{
                        color: "var(--color-text-muted)",
                        fontSize: "0.8125rem",
                      }}
                    >
                      {field.label}
                    </span>
                    <span style={{ fontWeight: 600 }}>{display}</span>
                  </span>
                );
              })}
            </div>
          </>
        )}
      </div>
    </div>
  );
}
