import { create } from "zustand";
import { Campaign, Message } from "../domain/campaign";
import * as campaignService from "../services/campaignService";
import * as llmService from "../services/llmService";

interface CampaignStore {
  campaigns: Campaign[];
  activeCampaignId: string | null;
  messages: Message[];
  isSending: boolean;
  error: string | null;

  loadCampaigns: () => Promise<void>;
  selectCampaign: (id: string) => Promise<void>;
  createCampaign: (name: string, rpgSystemId: string) => Promise<Campaign>;
  archiveCampaign: (id: string) => Promise<void>;
  sendMessage: (content: string, providerId: string, modelId: string) => Promise<void>;
  clearError: () => void;
}

export const useCampaignStore = create<CampaignStore>((set, get) => ({
  campaigns: [],
  activeCampaignId: null,
  messages: [],
  isSending: false,
  error: null,

  loadCampaigns: async () => {
    try {
      const campaigns = await campaignService.listCampaigns();
      set({ campaigns });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  selectCampaign: async (id: string) => {
    try {
      const messages = await campaignService.getMessages(id);
      set({ activeCampaignId: id, messages });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  createCampaign: async (name: string, rpgSystemId: string) => {
    const campaign = await campaignService.createCampaign(name, rpgSystemId);
    await get().loadCampaigns();
    await get().selectCampaign(campaign.id);
    return campaign;
  },

  archiveCampaign: async (id: string) => {
    await campaignService.archiveCampaign(id);
    if (get().activeCampaignId === id) {
      set({ activeCampaignId: null, messages: [] });
    }
    await get().loadCampaigns();
  },

  sendMessage: async (content: string, providerId: string, modelId: string) => {
    const { activeCampaignId } = get();
    if (!activeCampaignId) return;

    const userMsg: Message = {
      id: crypto.randomUUID(),
      campaign_id: activeCampaignId,
      role: "user",
      content,
      created_at: new Date().toISOString(),
    };

    set((state) => ({
      messages: [...state.messages, userMsg],
      isSending: true,
      error: null,
    }));

    try {
      const response = await llmService.sendChatMessage(activeCampaignId, content, providerId, modelId);
      const assistantMsg: Message = {
        id: crypto.randomUUID(),
        campaign_id: activeCampaignId,
        role: "assistant",
        content: response.content,
        created_at: new Date().toISOString(),
      };
      set((state) => ({
        messages: [...state.messages, assistantMsg],
        isSending: false,
      }));
    } catch (e) {
      set({ isSending: false, error: String(e) });
    }
  },

  clearError: () => set({ error: null }),
}));
