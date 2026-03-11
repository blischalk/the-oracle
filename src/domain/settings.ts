export interface AppSettings {
  active_provider_id: string;
  active_model_id: string;
  theme: string;
  is_fullscreen: boolean;
  narration_enabled: boolean;
  narration_rate: number;
  narration_voice_uri: string;
  tts_provider: string;
  tts_openai_voice: string;
}

export const DEFAULT_SETTINGS: AppSettings = {
  active_provider_id: "anthropic",
  active_model_id: "claude-sonnet-4-6",
  theme: "ink",
  is_fullscreen: false,
  narration_enabled: false,
  narration_rate: 1.0,
  narration_voice_uri: "",
  tts_provider: "system",
  tts_openai_voice: "nova",
};
