import { useState } from "react";
import { useSettingsStore } from "../../stores/settingsStore";
import { validateApiKey } from "../../services/llmService";

interface Props {
  providerId: string;
}

export function ApiKeyForm({ providerId }: Props) {
  const [key, setKey] = useState("");
  const [status, setStatus] = useState<"idle" | "validating" | "valid" | "invalid">("idle");
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const { saveApiKey } = useSettingsStore();

  async function handleSave() {
    if (!key.trim()) return;
    setStatus("validating");
    setErrorMessage(null);
    try {
      const isValid = await validateApiKey(providerId, key.trim());
      if (isValid) {
        await saveApiKey(providerId, key.trim());
        setStatus("valid");
        setKey("");
      } else {
        setStatus("invalid");
        setErrorMessage("Invalid API key. Please check and try again.");
      }
    } catch (err) {
      setStatus("invalid");
      const message =
        err instanceof Error
          ? err.message
          : typeof err === "string"
            ? err
            : err && typeof err === "object" && "message" in err
              ? String((err as { message: unknown }).message)
              : "Invalid API key. Please check and try again.";
      setErrorMessage(message);
    }
  }

  return (
    <div>
      <div className="flex gap-3" style={{ alignItems: "stretch" }}>
        <input
          type="password"
          className="oracle-input flex-1 min-w-0 focus:outline-none"
          style={{
            backgroundColor: "var(--color-bg)",
            border: `1px solid ${status === "invalid" ? "#ff4444" : "var(--color-border)"}`,
            color: "var(--color-text)",
          }}
          placeholder="sk-..."
          value={key}
          onChange={(e) => { setKey(e.target.value); setStatus("idle"); setErrorMessage(null); }}
        />
        <button
          type="button"
          className="oracle-btn shrink-0"
          style={{
            backgroundColor: "var(--color-primary)",
            color: "var(--color-bg)",
            opacity: status === "validating" ? 0.5 : 1,
          }}
          onClick={handleSave}
          disabled={status === "validating"}
        >
          {status === "validating" ? "Checking..." : "Save"}
        </button>
      </div>
      {status === "valid" && (
        <p className="text-sm mt-2 text-green-400" style={{ lineHeight: "var(--line-height-normal)" }}>
          API key saved successfully.
        </p>
      )}
      {status === "invalid" && errorMessage && (
        <p className="text-sm mt-2 text-red-400" style={{ lineHeight: "var(--line-height-normal)" }}>
          {errorMessage}
        </p>
      )}
    </div>
  );
}
