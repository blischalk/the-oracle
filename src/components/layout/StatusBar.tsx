import { ModelBadge } from "../settings/ModelBadge";

export function StatusBar() {
  return (
    <div
      className="flex items-center justify-end px-4 py-1"
      style={{
        backgroundColor: "var(--color-surface)",
        borderTop: "1px solid var(--color-border)",
        minHeight: "28px",
      }}
    >
      <ModelBadge />
    </div>
  );
}
