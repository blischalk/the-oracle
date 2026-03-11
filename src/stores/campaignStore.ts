import { create } from "zustand";
import { Campaign, CampaignState, Message } from "../domain/campaign";
import { RpgSystem } from "../domain/rpgSystem";
import * as campaignService from "../services/campaignService";
import * as llmService from "../services/llmService";

const requestedGreetingCampaignIds = new Set<string>();

interface CampaignStore {
  campaigns: Campaign[];
  activeCampaignId: string | null;
  activeCampaign: Campaign | null;
  campaignState: CampaignState | null;
  activeRpgSystem: RpgSystem | null;
  messages: Message[];
  isSending: boolean;
  isRequestingGreeting: boolean;
  error: string | null;

  loadCampaigns: () => Promise<void>;
  selectCampaign: (id: string) => Promise<void>;
  createCampaign: (name: string, rpgSystemId: string) => Promise<Campaign>;
  updateCampaignName: (id: string, name: string) => Promise<void>;
  archiveCampaign: (id: string) => Promise<void>;
  deleteCampaign: (id: string) => Promise<void>;
  sendMessage: (content: string, providerId: string, modelId: string) => Promise<void>;
  requestGreeting: (providerId: string, modelId: string) => Promise<void>;
  extractCharacterData: (providerId: string, modelId: string) => Promise<void>;
  patchCharacterData: (patch: Record<string, unknown>) => Promise<void>;
  clearError: () => void;
}

export const useCampaignStore = create<CampaignStore>((set, get) => ({
  campaigns: [],
  activeCampaignId: null,
  activeCampaign: null,
  campaignState: null,
  activeRpgSystem: null,
  messages: [],
  isSending: false,
  isRequestingGreeting: false,
  error: null,

  loadCampaigns: async () => {
    try {
      const campaigns = await campaignService.listCampaigns();
      set({
        campaigns: Array.isArray(campaigns) ? campaigns : [],
        error: null,
      });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  selectCampaign: async (id: string) => {
    try {
      const campaign = await campaignService.getCampaign(id);
      const [messages, campaignState, rpgSystem] = await Promise.all([
        campaignService.getMessages(id),
        campaignService.getCampaignState(id),
        campaign
          ? campaignService.getRpgSystem(campaign.rpg_system_id)
          : Promise.resolve(null),
      ]);
      set({
        activeCampaignId: id,
        activeCampaign: campaign ?? null,
        messages,
        campaignState,
        activeRpgSystem: rpgSystem ?? null,
      });
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

  updateCampaignName: async (id: string, name: string) => {
    try {
      const idStr = typeof id === "string" ? id : String(id);
      await campaignService.updateCampaignName(idStr, name);
      await get().loadCampaigns();
      set({ error: null });
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      console.error("updateCampaignName failed:", e);
      set({ error: message });
    }
  },

  archiveCampaign: async (id: string) => {
    await campaignService.archiveCampaign(id);
    if (get().activeCampaignId === id) {
      set({
        activeCampaignId: null,
        activeCampaign: null,
        campaignState: null,
        activeRpgSystem: null,
        messages: [],
      });
    }
    await get().loadCampaigns();
  },

  deleteCampaign: async (id: string) => {
    try {
      const idStr = typeof id === "string" ? id : String(id);
      await campaignService.deleteCampaign(idStr);
      if (get().activeCampaignId === idStr) {
        set({
          activeCampaignId: null,
          activeCampaign: null,
          campaignState: null,
          activeRpgSystem: null,
          messages: [],
        });
      }
      await get().loadCampaigns();
      set({ error: null });
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      console.error("deleteCampaign failed:", e);
      set({ error: message });
    }
  },

  sendMessage: async (content: string, providerId: string, modelId: string) => {
    const { activeCampaignId, messages } = get();
    if (!activeCampaignId) return;

    const wasFirstExchange = messages.length === 0;

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
      const updatedState = await campaignService.getCampaignState(activeCampaignId);
      set((state) => ({
        messages: [...state.messages, assistantMsg],
        isSending: false,
        campaignState: updatedState,
      }));

      if (wasFirstExchange) {
        campaignService.suggestCampaignName(activeCampaignId, providerId, modelId).then(
          () => get().loadCampaigns(),
          () => {}
        );
      }
    } catch (e) {
      set({ isSending: false, error: String(e) });
    }
  },

  requestGreeting: async (providerId: string, modelId: string) => {
    const { activeCampaignId, messages } = get();
    if (!activeCampaignId || requestedGreetingCampaignIds.has(activeCampaignId)) return;

    // Only send a greeting for brand-new campaigns. Resuming an existing
    // campaign already has the full history visible — no LLM call needed.
    if (messages.length > 0) {
      requestedGreetingCampaignIds.add(activeCampaignId);
      return;
    }

    requestedGreetingCampaignIds.add(activeCampaignId);
    const kind: campaignService.GreetingKind = "new";
    set({ isRequestingGreeting: true, error: null });

    try {
      await campaignService.requestGmGreeting(activeCampaignId, kind, providerId, modelId);
      const [updated, updatedState] = await Promise.all([
        campaignService.getMessages(activeCampaignId),
        campaignService.getCampaignState(activeCampaignId),
      ]);
      set({ messages: updated, isRequestingGreeting: false, campaignState: updatedState });
    } catch (e) {
      requestedGreetingCampaignIds.delete(activeCampaignId);
      set({ isRequestingGreeting: false, error: String(e) });
    }
  },

  extractCharacterData: async (providerId: string, modelId: string) => {
    const { activeCampaignId } = get();
    if (!activeCampaignId) return;
    try {
      const updated = await campaignService.extractCharacterData(
        activeCampaignId,
        providerId,
        modelId
      );
      set({ campaignState: updated });
    } catch {
      // Non-fatal: profile stays with dashes
    }
  },

  patchCharacterData: async (patch: Record<string, unknown>) => {
    const { activeCampaignId } = get();
    if (!activeCampaignId) return;
    try {
      const updated = await campaignService.patchCharacterData(activeCampaignId, patch);
      set({ campaignState: updated });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  clearError: () => set({ error: null }),
}));
