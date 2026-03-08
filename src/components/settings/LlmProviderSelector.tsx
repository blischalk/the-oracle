import { useSettingsStore } from "../../stores/settingsStore";
import { ApiKeyForm } from "./ApiKeyForm";

export function LlmProviderSelector() {
  const { settings, providers, updateSettings } = useSettingsStore();

  const activeProvider = providers.find((p) => p.id === settings.active_provider_id);

  return (
    <div>
      <label className="block text-sm font-semibold mb-2" style={{ color: "var(--color-text)" }}>LLM Provider</label>
      <select
        className="w-full p-2 mb-4 text-sm focus:outline-none"
        style={{
          backgroundColor: "var(--color-bg)",
          border: "1px solid var(--color-border)",
          color: "var(--color-text)",
          borderRadius: "var(--border-radius)",
        }}
        value={settings.active_provider_id}
        onChange={(e) => updateSettings({ active_provider_id: e.target.value, active_model_id: providers.find(p => p.id === e.target.value)?.models[0]?.id ?? "" })}
      >
        {providers.map((p) => (
          <option key={p.id} value={p.id}>{p.display_name}</option>
        ))}
      </select>

      {activeProvider && (
        <>
          <label className="block text-sm font-semibold mb-2" style={{ color: "var(--color-text)" }}>Model</label>
          <select
            className="w-full p-2 mb-4 text-sm focus:outline-none"
            style={{
              backgroundColor: "var(--color-bg)",
              border: "1px solid var(--color-border)",
              color: "var(--color-text)",
              borderRadius: "var(--border-radius)",
            }}
            value={settings.active_model_id}
            onChange={(e) => updateSettings({ active_model_id: e.target.value })}
          >
            {activeProvider.models.map((m) => (
              <option key={m.id} value={m.id}>{m.display_name}</option>
            ))}
          </select>

          <label className="block text-sm font-semibold mb-2" style={{ color: "var(--color-text)" }}>API Key</label>
          <ApiKeyForm providerId={settings.active_provider_id} />
        </>
      )}
    </div>
  );
}
