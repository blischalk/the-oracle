import { useNarration } from "../../hooks/useNarration";
import { useSettingsStore } from "../../stores/settingsStore";

const OPENAI_VOICES = [
  { id: "nova",    label: "Nova — warm, natural (female)" },
  { id: "shimmer", label: "Shimmer — gentle, soft (female)" },
  { id: "alloy",   label: "Alloy — balanced, neutral" },
  { id: "fable",   label: "Fable — expressive, British (male)" },
  { id: "onyx",    label: "Onyx — deep, authoritative (male)" },
  { id: "echo",    label: "Echo — warm (male)" },
];

const subLabelStyle: React.CSSProperties = {
  display: "block",
  fontSize: "0.75rem",
  color: "var(--color-text-muted)",
  marginBottom: "var(--space-1)",
  textTransform: "uppercase",
  letterSpacing: "0.05em",
};

const rowStyle: React.CSSProperties = {
  display: "flex",
  flexDirection: "column",
  gap: "var(--space-4)",
};

export function NarrationSettings() {
  const { settings, updateSettings } = useSettingsStore();
  const { voices, speak } = useNarration();

  const isOpenAI = settings.tts_provider === "openai";

  function handleToggle() {
    updateSettings({ narration_enabled: !settings.narration_enabled });
  }

  function handleProviderChange(e: React.ChangeEvent<HTMLSelectElement>) {
    updateSettings({ tts_provider: e.target.value });
  }

  function handleOpenAIVoiceChange(e: React.ChangeEvent<HTMLSelectElement>) {
    updateSettings({ tts_openai_voice: e.target.value });
  }

  function handleSystemVoiceChange(e: React.ChangeEvent<HTMLSelectElement>) {
    updateSettings({ narration_voice_uri: e.target.value });
  }

  function handleRateChange(e: React.ChangeEvent<HTMLInputElement>) {
    updateSettings({ narration_rate: parseFloat(e.target.value) });
  }

  function handleTest() {
    speak("The torchlight gutters. Somewhere ahead, something stirs in the dark.");
  }

  return (
    <div>
      <h3
        className="oracle-label"
        style={{ marginBottom: "var(--space-3)", display: "block" }}
      >
        Narration
      </h3>

      <label
        style={{
          display: "flex",
          alignItems: "center",
          gap: "var(--space-3)",
          cursor: "pointer",
          fontFamily: "var(--font-body)",
          fontSize: "0.9375rem",
          color: "var(--color-text)",
        }}
      >
        <input
          type="checkbox"
          checked={settings.narration_enabled}
          onChange={handleToggle}
        />
        Read GM responses aloud
      </label>

      {settings.narration_enabled && (
        <div style={{ ...rowStyle, marginTop: "var(--space-4)" }}>
          <div>
            <span style={subLabelStyle}>Voice engine</span>
            <select
              className="oracle-input w-full"
              value={settings.tts_provider}
              onChange={handleProviderChange}
            >
              <option value="system">System (built-in, free)</option>
              <option value="openai">OpenAI — high quality neural voices</option>
            </select>
          </div>

          {isOpenAI ? (
            <>
              <div
                style={{
                  padding: "var(--space-3)",
                  borderRadius: "var(--border-radius)",
                  backgroundColor: "var(--color-surface)",
                  border: "1px solid var(--color-border)",
                  fontSize: "0.8125rem",
                  color: "var(--color-text-muted)",
                  lineHeight: 1.5,
                }}
              >
                Requires an OpenAI API key saved in Settings → LLM Provider.
                Uses the <code>tts-1-hd</code> model (~$15 / 1M characters).
              </div>
              <div>
                <span style={subLabelStyle}>Voice</span>
                <select
                  className="oracle-input w-full"
                  value={settings.tts_openai_voice}
                  onChange={handleOpenAIVoiceChange}
                >
                  {OPENAI_VOICES.map((v) => (
                    <option key={v.id} value={v.id}>
                      {v.label}
                    </option>
                  ))}
                </select>
              </div>
            </>
          ) : (
            <div>
              <span style={subLabelStyle}>Voice</span>
              <select
                className="oracle-input w-full"
                value={settings.narration_voice_uri}
                onChange={handleSystemVoiceChange}
              >
                <option value="">System default</option>
                {voices.map((v) => (
                  <option key={v.voiceURI} value={v.voiceURI}>
                    {v.name}
                  </option>
                ))}
              </select>
            </div>
          )}

          <div>
            <span style={subLabelStyle}>
              Speed — {settings.narration_rate.toFixed(2)}×
            </span>
            <input
              type="range"
              min="0.5"
              max="2.0"
              step="0.05"
              value={settings.narration_rate}
              onChange={handleRateChange}
              style={{ width: "100%" }}
            />
          </div>

          <button
            type="button"
            className="oracle-btn oracle-btn-secondary"
            onClick={handleTest}
          >
            Test voice
          </button>
        </div>
      )}
    </div>
  );
}
