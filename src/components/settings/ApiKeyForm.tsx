import { useState } from "react";
import { useSettingsStore } from "../../stores/settingsStore";
import { validateApiKey } from "../../services/llmService";

interface Props {
  providerId: string;
}

export function ApiKeyForm({ providerId }: Props) {
  const [key, setKey] = useState("");
  const [status, setStatus] = useState<"idle" | "validating" | "valid" | "invalid">("idle");
  const { saveApiKey } = useSettingsStore();

  async function handleSave() {
    if (!key.trim()) return;
    setStatus("validating");
    try {
      const isValid = await validateApiKey(providerId, key.trim());
      if (isValid) {
        await saveApiKey(providerId, key.trim());
        setStatus("valid");
        setKey("");
      } else {
        setStatus("invalid");
      }
    } catch {
      setStatus("invalid");
    }
  }

  return (
    <div>
      <div className="flex gap-2">
        <input
          type="password"
          className="flex-1 p-2 text-sm focus:outline-none"
          style={{
            backgroundColor: "var(--color-bg)",
            border: `1px solid ${status === "invalid" ? "#ff4444" : "var(--color-border)"}`,
            color: "var(--color-text)",
            borderRadius: "var(--border-radius)",
          }}
          placeholder="sk-..."
          value={key}
          onChange={(e) => { setKey(e.target.value); setStatus("idle"); }}
        />
        <button
          className="px-3 py-2 text-sm font-semibold"
          style={{
            backgroundColor: "var(--color-primary)",
            color: "var(--color-bg)",
            borderRadius: "var(--border-radius)",
            opacity: status === "validating" ? 0.5 : 1,
          }}
          onClick={handleSave}
          disabled={status === "validating"}
        >
          {status === "validating" ? "Checking..." : "Save"}
        </button>
      </div>
      {status === "valid" && <p className="text-xs mt-1 text-green-400">API key saved successfully.</p>}
      {status === "invalid" && <p className="text-xs mt-1 text-red-400">Invalid API key. Please check and try again.</p>}
    </div>
  );
}
