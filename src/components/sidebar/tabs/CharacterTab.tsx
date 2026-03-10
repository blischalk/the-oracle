import { useCampaignStore } from "../../../stores/campaignStore";
import { CharacterField } from "../../../domain/rpgSystem";
import { getFieldTab } from "../fieldTabHelper";

const labelStyle: React.CSSProperties = {
  fontSize: "0.6875rem",
  color: "var(--color-text-muted)",
  textTransform: "uppercase",
  letterSpacing: "0.07em",
};

const valueStyle: React.CSSProperties = {
  fontSize: "0.9375rem",
  color: "var(--color-text)",
  fontWeight: 500,
  wordBreak: "break-word",
};

const sectionHeadingStyle: React.CSSProperties = {
  fontSize: "0.6875rem",
  color: "var(--color-text-muted)",
  textTransform: "uppercase",
  letterSpacing: "0.08em",
  borderBottom: "1px solid var(--color-border)",
  paddingBottom: "var(--space-1)",
  marginBottom: "var(--space-2)",
};

function fieldValue(characterData: Record<string, unknown>, field: CharacterField): string {
  const value = characterData[field.name];
  return value !== undefined && value !== null && value !== "" ? String(value) : "—";
}

interface IdentityRowProps {
  field: CharacterField;
  characterData: Record<string, unknown>;
}

function IdentityRow({ field, characterData }: IdentityRowProps) {
  return (
    <div style={{ marginBottom: "var(--space-3)" }}>
      <div style={labelStyle}>{field.label}</div>
      <div style={valueStyle}>{fieldValue(characterData, field)}</div>
    </div>
  );
}

interface StatCellProps {
  field: CharacterField;
  characterData: Record<string, unknown>;
}

function StatCell({ field, characterData }: StatCellProps) {
  return (
    <div
      style={{
        padding: "var(--space-2) var(--space-2)",
        border: "1px solid var(--color-border)",
        borderRadius: "var(--border-radius)",
        backgroundColor: "var(--color-surface)",
        textAlign: "center",
      }}
    >
      <div style={labelStyle}>{field.label}</div>
      <div
        style={{
          fontSize: "1.25rem",
          fontWeight: 700,
          color: "var(--color-text)",
          lineHeight: 1.2,
          marginTop: "2px",
        }}
      >
        {fieldValue(characterData, field)}
      </div>
    </div>
  );
}

export function CharacterTab() {
  const { activeCampaignId, activeRpgSystem, campaignState } = useCampaignStore();

  if (!activeCampaignId || !activeRpgSystem) {
    return (
      <div style={{ padding: "var(--space-4)", color: "var(--color-text-muted)", fontSize: "0.875rem" }}>
        Select a campaign to view character details.
      </div>
    );
  }

  const characterData = (campaignState?.character_data ?? {}) as Record<string, unknown>;
  const characterFields = activeRpgSystem.character_fields.filter(
    (field) => getFieldTab(field.name) === "character"
  );

  if (characterFields.length === 0) {
    return (
      <div style={{ padding: "var(--space-4)", color: "var(--color-text-muted)", fontSize: "0.875rem" }}>
        No character fields defined.
      </div>
    );
  }

  const identityFields = characterFields.filter((f) => f.field_type.type !== "Number");
  const statFields = characterFields.filter((f) => f.field_type.type === "Number");

  return (
    <div className="overflow-y-auto" style={{ padding: "var(--space-3)" }}>
      {identityFields.length > 0 && (
        <section style={{ marginBottom: statFields.length > 0 ? "var(--space-4)" : 0 }}>
          {identityFields.map((field) => (
            <IdentityRow key={field.name} field={field} characterData={characterData} />
          ))}
        </section>
      )}

      {statFields.length > 0 && (
        <section>
          <div style={sectionHeadingStyle}>Stats</div>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(2, 1fr)",
              gap: "var(--space-2)",
            }}
          >
            {statFields.map((field) => (
              <StatCell key={field.name} field={field} characterData={characterData} />
            ))}
          </div>
        </section>
      )}
    </div>
  );
}
