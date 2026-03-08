import { useSettingsStore } from "../../stores/settingsStore";
import { ApiKeyForm } from "./ApiKeyForm";
import { ThemedSelect } from "../ui/ThemedSelect";

export function LlmProviderSelector() {
  const { settings, providers, updateSettings } = useSettingsStore();

  const activeProvider = providers.find((p) => p.id === settings.active_provider_id);

  return (
    <div>
      <div className="oracle-form-group">
        <label className="oracle-label" htmlFor="llm-provider">LLM Provider</label>
        <ThemedSelect
          id="llm-provider"
          options={providers.map((p) => ({ value: p.id, label: p.display_name }))}
          value={settings.active_provider_id}
          onChange={(val) => updateSettings({ active_provider_id: val, active_model_id: providers.find(p => p.id === val)?.models[0]?.id ?? "" })}
        />
      </div>

      {activeProvider && (
        <>
          <div className="oracle-form-group">
            <label className="oracle-label" htmlFor="model">Model</label>
            <ThemedSelect
              id="model"
              options={activeProvider.models.map((m) => ({ value: m.id, label: m.display_name }))}
              value={settings.active_model_id}
              onChange={(val) => updateSettings({ active_model_id: val })}
            />
          </div>

          <div className="oracle-form-group">
            <label className="oracle-label" htmlFor="api-key">API Key</label>
            <ApiKeyForm providerId={settings.active_provider_id} />
          </div>
        </>
      )}
    </div>
  );
}
