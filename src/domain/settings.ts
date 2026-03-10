export interface AppSettings {
  active_provider_id: string;
  active_model_id: string;
  theme: string;
  is_fullscreen: boolean;
}

export const DEFAULT_SETTINGS: AppSettings = {
  active_provider_id: "anthropic",
  active_model_id: "claude-sonnet-4-6",
  theme: "ink",
  is_fullscreen: false,
};
