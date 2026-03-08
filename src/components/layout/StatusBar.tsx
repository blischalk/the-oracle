import { ModelBadge } from "../settings/ModelBadge";

export function StatusBar() {
  return (
    <div
      className="flex items-center justify-end"
      style={{
        padding: "var(--space-2) var(--space-4)",
        backgroundColor: "var(--color-surface)",
        borderTop: "1px solid var(--color-border)",
        minHeight: "2rem",
      }}
    >
      <ModelBadge />
    </div>
  );
}
