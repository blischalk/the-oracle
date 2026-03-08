import { invoke } from "@tauri-apps/api/core";
import { Campaign, Message } from "../domain/campaign";

export async function listCampaigns(): Promise<Campaign[]> {
  return invoke("list_campaigns");
}

export async function createCampaign(name: string, rpg_system_id: string): Promise<Campaign> {
  return invoke("create_campaign", { name, rpg_system_id });
}

export async function getCampaign(id: string): Promise<Campaign | null> {
  return invoke("get_campaign", { id });
}

export async function archiveCampaign(id: string): Promise<void> {
  return invoke("archive_campaign", { id });
}

export async function getMessages(campaign_id: string): Promise<Message[]> {
  return invoke("get_messages", { campaign_id });
}
