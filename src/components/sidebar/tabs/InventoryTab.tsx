import { useCampaignStore } from "../../../stores/campaignStore";
import { getFieldTab } from "../fieldTabHelper";

// Splits a raw inventory string (comma- or newline-separated) into individual
// item strings, discarding blanks and degenerate entries.
function parseItems(raw: string): string[] {
  return raw
    .split(/[\n,]+/)
    .map((s) => s.trim())
    .filter((s) => s.length > 0 && s !== "—");
}

// Collects every inventory field's value into a flat item list.
function collectItems(
  fields: { name: string }[],
  characterData: Record<string, unknown>
): string[] {
  return fields.flatMap((field) => {
    const value = characterData[field.name];
    if (!value || typeof value !== "string") return [];
    return parseItems(value);
  });
}

export function InventoryTab() {
  const { activeCampaignId, activeRpgSystem, campaignState } = useCampaignStore();

  if (!activeCampaignId || !activeRpgSystem) {
    return (
      <div style={{ padding: "var(--space-4)", color: "var(--color-text-muted)", fontSize: "0.875rem" }}>
        Select a campaign to view inventory.
      </div>
    );
  }

  const characterData = (campaignState?.character_data ?? {}) as Record<string, unknown>;
  const inventoryFields = activeRpgSystem.character_fields.filter(
    (field) => getFieldTab(field.name) === "inventory"
  );

  if (inventoryFields.length === 0) {
    return (
      <div style={{ padding: "var(--space-4)", color: "var(--color-text-muted)", fontSize: "0.875rem" }}>
        No inventory fields defined for this system.
      </div>
    );
  }

  const items = collectItems(inventoryFields, characterData);

  return (
    <div className="overflow-y-auto" style={{ padding: "var(--space-3)" }}>
      {items.length === 0 ? (
        <div style={{ color: "var(--color-text-muted)", fontSize: "0.875rem" }}>
          No items yet.
        </div>
      ) : (
        <ul style={{ listStyle: "none", margin: 0, padding: 0 }}>
          {items.map((item, i) => (
            <li
              key={i}
              style={{
                display: "flex",
                alignItems: "baseline",
                gap: "var(--space-2)",
                padding: "var(--space-2) 0",
                borderBottom: "1px solid var(--color-border)",
                fontSize: "0.9375rem",
                color: "var(--color-text)",
              }}
            >
              <span
                aria-hidden
                style={{
                  flexShrink: 0,
                  width: "6px",
                  height: "6px",
                  borderRadius: "50%",
                  backgroundColor: "var(--color-text-muted)",
                  marginTop: "6px",
                  alignSelf: "flex-start",
                }}
              />
              {item}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
