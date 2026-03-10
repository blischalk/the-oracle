import { useState } from "react";
import { useCampaignStore } from "../../../stores/campaignStore";
import type { StoryThread } from "../../../domain/campaign";

const STATUS_LABELS: Record<StoryThread["status"], string> = {
  active: "Active",
  potential: "Potential",
  completed: "Completed",
};

const STATUS_CYCLE: Record<StoryThread["status"], StoryThread["status"]> = {
  active: "completed",
  potential: "active",
  completed: "potential",
};

function ThreadRow({
  thread,
  onCycleStatus,
  onDelete,
}: {
  thread: StoryThread;
  onCycleStatus: () => void;
  onDelete: () => void;
}) {
  const isCompleted = thread.status === "completed";
  return (
    <div
      style={{
        padding: "var(--space-2) var(--space-3)",
        borderRadius: "var(--border-radius)",
        border: "1px solid var(--color-border)",
        marginBottom: "var(--space-2)",
        opacity: isCompleted ? 0.6 : 1,
      }}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="flex-1 min-w-0">
          <div style={{ fontWeight: 600, fontSize: "0.875rem", color: "var(--color-text)" }}>
            {thread.title}
            <span
              style={{
                marginLeft: "var(--space-2)",
                fontSize: "0.75rem",
                color: "var(--color-text-muted)",
                fontWeight: 400,
              }}
            >
              {STATUS_LABELS[thread.status]}
            </span>
          </div>
          {thread.description && (
            <div style={{ fontSize: "0.8125rem", color: "var(--color-text-muted)", marginTop: "2px", wordBreak: "break-word" }}>
              {thread.description}
            </div>
          )}
        </div>
        <div className="flex gap-1 flex-shrink-0">
          <button
            type="button"
            onClick={onCycleStatus}
            title="Cycle status"
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
            ↻
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

export function QuestsTab() {
  const { activeCampaignId, campaignState, patchCharacterData } = useCampaignStore();
  const [isAdding, setIsAdding] = useState(false);
  const [newTitle, setNewTitle] = useState("");
  const [newDescription, setNewDescription] = useState("");
  const [newStatus, setNewStatus] = useState<StoryThread["status"]>("active");

  if (!activeCampaignId) {
    return (
      <div style={{ padding: "var(--space-4)", color: "var(--color-text-muted)", fontSize: "0.875rem" }}>
        Select a campaign to view story threads.
      </div>
    );
  }

  const characterData = (campaignState?.character_data ?? {}) as Record<string, unknown>;
  const threads: StoryThread[] = Array.isArray(characterData.__story_threads)
    ? (characterData.__story_threads as StoryThread[])
    : [];

  function saveThreads(updated: StoryThread[]) {
    patchCharacterData({ __story_threads: updated });
  }

  function handleAdd() {
    if (!newTitle.trim()) return;
    const thread: StoryThread = {
      id: crypto.randomUUID(),
      title: newTitle.trim(),
      description: newDescription.trim(),
      status: newStatus,
    };
    saveThreads([...threads, thread]);
    setNewTitle("");
    setNewDescription("");
    setNewStatus("active");
    setIsAdding(false);
  }

  function handleCycleStatus(id: string) {
    const updated = threads.map((t) =>
      t.id === id ? { ...t, status: STATUS_CYCLE[t.status] } : t
    );
    saveThreads(updated);
  }

  function handleDelete(id: string) {
    saveThreads(threads.filter((t) => t.id !== id));
  }

  const grouped = {
    active: threads.filter((t) => t.status === "active"),
    potential: threads.filter((t) => t.status === "potential"),
    completed: threads.filter((t) => t.status === "completed"),
  };

  return (
    <div className="flex flex-col h-full overflow-hidden">
      <div className="flex-1 overflow-y-auto" style={{ padding: "var(--space-3)" }}>
        {threads.length === 0 && !isAdding && (
          <p style={{ color: "var(--color-text-muted)", fontSize: "0.875rem", textAlign: "center", marginTop: "var(--space-4)" }}>
            No story threads yet.
          </p>
        )}
        {(["active", "potential", "completed"] as StoryThread["status"][]).map((status) => {
          const group = grouped[status];
          if (group.length === 0) return null;
          return (
            <div key={status}>
              <div style={{ fontSize: "0.75rem", color: "var(--color-text-muted)", textTransform: "uppercase", letterSpacing: "0.05em", marginBottom: "var(--space-2)" }}>
                {STATUS_LABELS[status]}
              </div>
              {group.map((thread) => (
                <ThreadRow
                  key={thread.id}
                  thread={thread}
                  onCycleStatus={() => handleCycleStatus(thread.id)}
                  onDelete={() => handleDelete(thread.id)}
                />
              ))}
            </div>
          );
        })}
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
                placeholder="Title"
                value={newTitle}
                onChange={(e) => setNewTitle(e.target.value)}
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
                value={newStatus}
                onChange={(e) => setNewStatus(e.target.value as StoryThread["status"])}
                style={{
                  padding: "var(--space-1) var(--space-2)",
                  borderRadius: "var(--border-radius)",
                  border: "1px solid var(--color-border)",
                  background: "var(--color-bg)",
                  color: "var(--color-text)",
                  fontSize: "0.875rem",
                }}
              >
                <option value="active">Active</option>
                <option value="potential">Potential</option>
                <option value="completed">Completed</option>
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
                onClick={() => { setIsAdding(false); setNewTitle(""); setNewDescription(""); }}
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
            + Add Thread
          </button>
        </div>
      )}
    </div>
  );
}
