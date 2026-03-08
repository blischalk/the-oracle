import { invoke } from "@tauri-apps/api/core";
import { LlmResponse, ProviderInfo } from "../domain/llmProvider";

export async function sendChatMessage(
  campaign_id: string,
  user_message: string,
  provider_id: string,
  model_id: string
): Promise<LlmResponse> {
  return invoke("send_chat_message", { campaign_id, user_message, provider_id, model_id });
}

export async function listProviders(): Promise<ProviderInfo[]> {
  return invoke("list_providers");
}

export async function validateApiKey(provider_id: string, key: string): Promise<boolean> {
  return invoke("validate_api_key", { provider_id, key });
}
