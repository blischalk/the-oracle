import { invoke } from "@tauri-apps/api/core";
import type { Campaign, CampaignState, Message } from "../domain/campaign";
import { RpgSystem } from "../domain/rpgSystem";

export async function listCampaigns(): Promise<Campaign[]> {
  return invoke("list_campaigns");
}

export async function createCampaign(name: string, rpg_system_id: string): Promise<Campaign> {
  return invoke("create_campaign", { name, rpgSystemId: rpg_system_id });
}

export async function getCampaign(id: string): Promise<Campaign | null> {
  return invoke("get_campaign", { id });
}

export async function updateCampaignName(id: string, name: string): Promise<void> {
  return invoke("update_campaign_name", { id, name });
}

export async function suggestCampaignName(
  campaignId: string,
  providerId: string,
  modelId: string
): Promise<void> {
  return invoke("suggest_campaign_name", {
    campaignId,
    providerId,
    modelId,
  });
}

export async function archiveCampaign(id: string): Promise<void> {
  return invoke("archive_campaign", { id });
}

export async function deleteCampaign(id: string): Promise<void> {
  return invoke("delete_campaign", { id });
}

export async function getMessages(campaign_id: string): Promise<Message[]> {
  return invoke("get_messages", { campaignId: campaign_id });
}

export async function getCampaignState(campaignId: string): Promise<CampaignState> {
  return invoke("get_campaign_state", { campaignId });
}

export async function getRpgSystem(id: string): Promise<RpgSystem | null> {
  return invoke("get_rpg_system", { id });
}

export async function listRpgSystems(): Promise<RpgSystem[]> {
  return invoke("list_rpg_systems");
}

export async function openUserSystemsFolder(): Promise<string> {
  return invoke("open_user_systems_folder");
}

export async function extractCharacterData(
  campaignId: string,
  providerId: string,
  modelId: string
): Promise<CampaignState> {
  return invoke("extract_character_data", {
    campaignId,
    providerId,
    modelId,
  });
}

export type GreetingKind = "new" | "resume";

export async function requestGmGreeting(
  campaignId: string,
  kind: GreetingKind,
  providerId: string,
  modelId: string
): Promise<{ content: string }> {
  return invoke("request_gm_greeting", {
    campaignId,
    kind,
    providerId,
    modelId,
  });
}

export async function patchCharacterData(
  campaignId: string,
  patch: Record<string, unknown>
): Promise<CampaignState> {
  return invoke("patch_character_data", { campaignId, patch });
}

export function exportCampaignToMarkdown(campaign: Campaign, messages: Message[]): string {
  const header = `# ${campaign.name}\n\n**RPG System:** ${campaign.rpg_system_id}\n**Created:** ${new Date(campaign.created_at).toLocaleDateString()}\n\n---\n\n`;

  const body = messages
    .filter((m) => m.role !== "system")
    .map((m) => {
      const speaker = m.role === "user" ? "**You**" : "**The Oracle**";
      return `${speaker}\n\n${m.content}`;
    })
    .join("\n\n---\n\n");

  return header + body;
}
