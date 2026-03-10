import { useState } from "react";
import { useCampaignStore } from "../../../stores/campaignStore";
import type { NpcEntry } from "../../../domain/campaign";

function EmptyMessage({ text }: { text: string }) {
  return (
    <p style={{ color: "var(--color-text-muted)", fontSize: "0.875rem", textAlign: "center", marginTop: "var(--space-4)" }}>
      {text}
    </p>
  );
}

function NpcRow({
  entry,
  onToggleStatus,
  onDelete,
}: {
  entry: NpcEntry;
  onToggleStatus: () => void;
  onDelete: () => void;
}) {
  const isPast = entry.status === "past";
  return (
    <div
      style={{
        padding: "var(--space-2) var(--space-3)",
        borderRadius: "var(--border-radius)",
        border: "1px solid var(--color-border)",
        marginBottom: "var(--space-2)",
        opacity: isPast ? 0.6 : 1,
      }}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="flex-1 min-w-0">
          <div style={{ fontWeight: 600, fontSize: "0.875rem", color: "var(--color-text)" }}>
            {entry.name}
            <span
              style={{
                marginLeft: "var(--space-2)",
                fontSize: "0.75rem",
                color: "var(--color-text-muted)",
                fontWeight: 400,
              }}
            >
              {entry.type}
            </span>
          </div>
          {entry.description && (
            <div style={{ fontSize: "0.8125rem", color: "var(--color-text-muted)", marginTop: "2px", wordBreak: "break-word" }}>
              {entry.description}
            </div>
          )}
        </div>
        <div className="flex gap-1 flex-shrink-0">
          <button
            type="button"
            onClick={onToggleStatus}
            title={isPast ? "Mark active" : "Mark past"}
            style={{
              fontSize: "0.75rem",
              padding: "2px 6px",
              borderRadius: "var(--border-radius)",
              border: "1px solid var(--color-border)",
              color: "var(--color-text-muted)",
              background: "none",
              cursor: "pointer",
            }}
          >
            {isPast ? "↩" : "✓"}
          </button>
          <button
            type="button"
            onClick={onDelete}
            title="Delete"
            style={{
              fontSize: "0.75rem",
              padding: "2px 6px",
              borderRadius: "var(--border-radius)",
              border: "1px solid var(--color-border)",
              color: "var(--color-error, #b91c1c)",
              background: "none",
              cursor: "pointer",
            }}
          >
            ✕
          </button>
        </div>
      </div>
    </div>
  );
}

export function JournalTab() {
  const { activeCampaignId, campaignState, patchCharacterData } = useCampaignStore();
  const [isAdding, setIsAdding] = useState(false);
  const [newName, setNewName] = useState("");
  const [newDescription, setNewDescription] = useState("");
  const [newType, setNewType] = useState<"npc" | "location">("npc");

  if (!activeCampaignId) {
    return (
      <div style={{ padding: "var(--space-4)", color: "var(--color-text-muted)", fontSize: "0.875rem" }}>
        Select a campaign to view the journal.
      </div>
    );
  }

  const characterData = (campaignState?.character_data ?? {}) as Record<string, unknown>;
  const entries: NpcEntry[] = Array.isArray(characterData.__npcs)
    ? (characterData.__npcs as NpcEntry[])
    : [];

  function saveEntries(updated: NpcEntry[]) {
    patchCharacterData({ __npcs: updated });
  }

  function handleAdd() {
    if (!newName.trim()) return;
    const entry: NpcEntry = {
      id: crypto.randomUUID(),
      name: newName.trim(),
      description: newDescription.trim(),
      type: newType,
      status: "active",
    };
    saveEntries([...entries, entry]);
    setNewName("");
    setNewDescription("");
    setNewType("npc");
    setIsAdding(false);
  }

  function handleToggleStatus(id: string) {
    const updated = entries.map((e) =>
      e.id === id ? { ...e, status: e.status === "active" ? "past" : "active" } as NpcEntry : e
    );
    saveEntries(updated);
  }

  function handleDelete(id: string) {
    saveEntries(entries.filter((e) => e.id !== id));
  }

  const active = entries.filter((e) => e.status === "active");
  const past = entries.filter((e) => e.status === "past");

  return (
    <div className="flex flex-col h-full overflow-hidden">
      <div className="flex-1 overflow-y-auto" style={{ padding: "var(--space-3)" }}>
        {entries.length === 0 && !isAdding && (
          <EmptyMessage text="No NPCs or locations yet." />
        )}
        {active.length > 0 && (
          <>
            <div style={{ fontSize: "0.75rem", color: "var(--color-text-muted)", textTransform: "uppercase", letterSpacing: "0.05em", marginBottom: "var(--space-2)" }}>
              Active
            </div>
            {active.map((entry) => (
              <NpcRow
                key={entry.id}
                entry={entry}
                onToggleStatus={() => handleToggleStatus(entry.id)}
                onDelete={() => handleDelete(entry.id)}
              />
            ))}
          </>
        )}
        {past.length > 0 && (
          <>
            <div style={{ fontSize: "0.75rem", color: "var(--color-text-muted)", textTransform: "uppercase", letterSpacing: "0.05em", marginTop: "var(--space-3)", marginBottom: "var(--space-2)" }}>
              Past
            </div>
            {past.map((entry) => (
              <NpcRow
                key={entry.id}
                entry={entry}
                onToggleStatus={() => handleToggleStatus(entry.id)}
                onDelete={() => handleDelete(entry.id)}
              />
            ))}
          </>
        )}
        {isAdding && (
          <div
            style={{
              padding: "var(--space-3)",
              borderRadius: "var(--border-radius)",
              border: "1px solid var(--color-primary)",
              marginTop: "var(--space-2)",
            }}
          >
            <div className="flex gap-2 mb-2">
              <input
                autoFocus
                type="text"
                placeholder="Name"
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && handleAdd()}
                style={{
                  flex: 1,
                  padding: "var(--space-1) var(--space-2)",
                  borderRadius: "var(--border-radius)",
                  border: "1px solid var(--color-border)",
                  background: "var(--color-bg)",
                  color: "var(--color-text)",
                  fontSize: "0.875rem",
                }}
              />
              <select
                value={newType}
                onChange={(e) => setNewType(e.target.value as "npc" | "location")}
                style={{
                  padding: "var(--space-1) var(--space-2)",
                  borderRadius: "var(--border-radius)",
                  border: "1px solid var(--color-border)",
                  background: "var(--color-bg)",
                  color: "var(--color-text)",
                  fontSize: "0.875rem",
                }}
              >
                <option value="npc">NPC</option>
                <option value="location">Location</option>
              </select>
            </div>
            <textarea
              placeholder="Description (optional)"
              value={newDescription}
              onChange={(e) => setNewDescription(e.target.value)}
              rows={2}
              style={{
                width: "100%",
                padding: "var(--space-1) var(--space-2)",
                borderRadius: "var(--border-radius)",
                border: "1px solid var(--color-border)",
                background: "var(--color-bg)",
                color: "var(--color-text)",
                fontSize: "0.875rem",
                resize: "none",
                marginBottom: "var(--space-2)",
              }}
            />
            <div className="flex gap-2">
              <button
                type="button"
                onClick={handleAdd}
                className="oracle-btn"
                style={{ backgroundColor: "var(--color-primary)", color: "var(--color-bg)", flex: 1 }}
              >
                Add
              </button>
              <button
                type="button"
                onClick={() => { setIsAdding(false); setNewName(""); setNewDescription(""); }}
                className="oracle-btn oracle-btn-secondary"
                style={{ flex: 1 }}
              >
                Cancel
              </button>
            </div>
          </div>
        )}
      </div>
      {!isAdding && (
        <div style={{ padding: "var(--space-3)", borderTop: "1px solid var(--color-border)" }}>
          <button
            type="button"
            className="oracle-btn w-full"
            style={{ backgroundColor: "var(--color-primary)", color: "var(--color-bg)" }}
            onClick={() => setIsAdding(true)}
          >
            + Add Entry
          </button>
        </div>
      )}
    </div>
  );
}
