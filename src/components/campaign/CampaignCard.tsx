import { useState, useRef, useEffect } from "react";
import { confirm } from "@tauri-apps/plugin-dialog";
import { Campaign } from "../../domain/campaign";


interface Props {
  campaign: Campaign;
  isActive: boolean;
  onSelect: () => void;
  onRename: (newName: string) => void;
  onDelete: () => void;
}

const buttonStyle = {
  color: "var(--color-text-muted)",
  fontSize: "0.8125rem",
  padding: "var(--space-1) 0",
  background: "none",
  border: "none",
  cursor: "pointer",
} as const;

const DELETE_CONFIRM_MESSAGE =
  "Permanently delete this campaign? All messages will be removed. This cannot be undone.";

export function CampaignCard({ campaign, isActive, onSelect, onRename, onDelete }: Props) {
  const [isRenameOpen, setIsRenameOpen] = useState(false);
  const [renameDraft, setRenameDraft] = useState(campaign.name);
  const renameInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    setRenameDraft(campaign.name);
  }, [campaign.name]);

  useEffect(() => {
    if (isRenameOpen) {
      setRenameDraft(campaign.name);
      renameInputRef.current?.focus();
    }
  }, [isRenameOpen, campaign.name]);

  function handleRenameClick(e: React.MouseEvent) {
    e.stopPropagation();
    setIsRenameOpen(true);
  }

  function handleRenameSave(e: React.MouseEvent) {
    e.stopPropagation();
    const trimmed = renameDraft.trim();
    if (trimmed) {
      onRename(trimmed);
      setIsRenameOpen(false);
    }
  }

  function handleRenameCancel(e: React.MouseEvent) {
    e.stopPropagation();
    setRenameDraft(campaign.name);
    setIsRenameOpen(false);
  }

  async function handleDeleteClick(e: React.MouseEvent) {
    e.stopPropagation();
    const ok = await confirm(DELETE_CONFIRM_MESSAGE, {
      title: "Delete campaign",
      kind: "warning",
      okLabel: "Delete",
      cancelLabel: "Cancel",
    });
    if (ok) onDelete();
  }

  return (
    <div
      className="cursor-pointer transition-all"
      style={{
        padding: "var(--space-3)",
        marginBottom: "var(--space-2)",
        backgroundColor: isActive ? "var(--color-border)" : "var(--color-surface)",
        border: `1px solid ${isActive ? "var(--color-primary)" : "var(--color-border)"}`,
        borderRadius: "var(--border-radius)",
      }}
      onClick={onSelect}
    >
      <p
        className="font-semibold truncate"
        style={{ color: "var(--color-text)", fontSize: "0.9375rem", lineHeight: "var(--line-height-tight)" }}
      >
        {campaign.name}
      </p>
      <p
        className="text-sm mt-1"
        style={{ color: "var(--color-text-muted)", lineHeight: "var(--line-height-normal)" }}
      >
        {campaign.rpg_system_id}
      </p>
      <div
        className="flex gap-2 mt-2"
        style={{ minWidth: 0 }}
        onClick={(e) => e.stopPropagation()}
      >
        {isRenameOpen ? (
          <>
            <input
              ref={renameInputRef}
              type="text"
              value={renameDraft}
              onChange={(e) => setRenameDraft(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  const trimmed = renameDraft.trim();
                  if (trimmed) {
                    onRename(trimmed);
                    setIsRenameOpen(false);
                  }
                }
                if (e.key === "Escape") {
                  setRenameDraft(campaign.name);
                  setIsRenameOpen(false);
                }
              }}
              style={{
                flex: "1 1 0",
                minWidth: 0,
                padding: "var(--space-1) var(--space-2)",
                fontSize: "0.8125rem",
                border: "1px solid var(--color-border)",
                borderRadius: "var(--border-radius)",
                color: "var(--color-text)",
                backgroundColor: "var(--color-bg)",
              }}
              aria-label="New campaign name"
            />
            <button
              type="button"
              className="opacity-60 hover:opacity-100 transition-opacity"
              style={{ ...buttonStyle, flexShrink: 0 }}
              onClick={handleRenameSave}
            >
              Save
            </button>
            <button
              type="button"
              className="opacity-60 hover:opacity-100 transition-opacity"
              style={{ ...buttonStyle, flexShrink: 0 }}
              onClick={handleRenameCancel}
            >
              Cancel
            </button>
          </>
        ) : (
          <>
            <button
              type="button"
              className="opacity-60 hover:opacity-100 transition-opacity"
              style={buttonStyle}
              onClick={handleRenameClick}
            >
              Rename
            </button>
            <button
              type="button"
              className="opacity-60 hover:opacity-100 transition-opacity"
              style={buttonStyle}
              onClick={handleDeleteClick}
            >
              Delete
            </button>
          </>
        )}
      </div>
    </div>
  );
}
