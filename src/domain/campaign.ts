export type MessageRole = "system" | "user" | "assistant";

export interface Message {
  id: string;
  campaign_id: string;
  role: MessageRole;
  content: string;
  created_at: string;
  token_count?: number;
}

export interface Campaign {
  id: string;
  name: string;
  rpg_system_id: string;
  created_at: string;
  updated_at: string;
  is_archived: boolean;
}

export interface CampaignState {
  campaign_id: string;
  character_data: Record<string, unknown>;
  notes: string;
  updated_at: string;
}

export interface NpcEntry {
  id: string;
  name: string;
  description: string;
  type: "npc" | "location";
  status: "active" | "past";
}

export interface StoryThread {
  id: string;
  title: string;
  description: string;
  status: "active" | "potential" | "completed";
}
