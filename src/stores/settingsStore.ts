import { create } from "zustand";
import { AppSettings, DEFAULT_SETTINGS } from "../domain/settings";
import { ProviderInfo } from "../domain/llmProvider";
import * as settingsService from "../services/settingsService";
import * as llmService from "../services/llmService";

interface SettingsStore {
  settings: AppSettings;
  providers: ProviderInfo[];
  isLoaded: boolean;

  loadSettings: () => Promise<void>;
  updateSettings: (partial: Partial<AppSettings>) => Promise<void>;
  saveApiKey: (providerId: string, key: string) => Promise<void>;
  getApiKey: (providerId: string) => Promise<string | null>;
  deleteApiKey: (providerId: string) => Promise<void>;
  loadProviders: () => Promise<void>;
}

export const useSettingsStore = create<SettingsStore>((set, get) => ({
  settings: DEFAULT_SETTINGS,
  providers: [],
  isLoaded: false,

  loadSettings: async () => {
    try {
      const settings = await settingsService.getSettings();
      set({ settings, isLoaded: true });
    } catch {
      set({ isLoaded: true });
    }
  },

  updateSettings: async (partial: Partial<AppSettings>) => {
    const next = { ...get().settings, ...partial };
    set({ settings: next });
    await settingsService.saveSettings(next);
  },

  saveApiKey: async (providerId: string, key: string) => {
    await settingsService.saveApiKey(providerId, key);
  },

  getApiKey: async (providerId: string) => {
    return settingsService.getApiKey(providerId);
  },

  deleteApiKey: async (providerId: string) => {
    await settingsService.deleteApiKey(providerId);
  },

  loadProviders: async () => {
    const providers = await llmService.listProviders();
    set({ providers });
  },
}));
