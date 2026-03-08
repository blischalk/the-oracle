import { useSettingsStore } from "../../stores/settingsStore";

export function ModelBadge() {
  const { settings, providers } = useSettingsStore();

  const provider = providers.find((p) => p.id === settings.active_provider_id);
  const model = provider?.models.find((m) => m.id === settings.active_model_id);

  if (!provider) return null;

  return (
    <span
      className="text-xs px-2 py-1 rounded"
      style={{
        backgroundColor: "var(--color-surface)",
        border: "1px solid var(--color-border)",
        color: "var(--color-text-muted)",
        borderRadius: "var(--border-radius)",
      }}
    >
      {provider.display_name} · {model?.display_name ?? settings.active_model_id}
    </span>
  );
}
