import { invoke } from "@tauri-apps/api/core";
import { AppSettings } from "../domain/settings";

export async function getSettings(): Promise<AppSettings> {
  return invoke("get_settings");
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke("save_settings", { settings });
}

export async function saveApiKey(provider_id: string, key: string): Promise<void> {
  return invoke("save_api_key", { providerId: provider_id, key });
}

export async function getApiKey(provider_id: string): Promise<string | null> {
  return invoke("get_api_key", { providerId: provider_id });
}

export async function deleteApiKey(provider_id: string): Promise<void> {
  return invoke("delete_api_key", { providerId: provider_id });
}
