import { create } from "zustand";

interface UiStore {
  isSidebarOpen: boolean;
  isSettingsOpen: boolean;
  isNewCampaignModalOpen: boolean;

  toggleSidebar: () => void;
  openSettings: () => void;
  closeSettings: () => void;
  openNewCampaignModal: () => void;
  closeNewCampaignModal: () => void;
}

export const useUiStore = create<UiStore>((set) => ({
  isSidebarOpen: true,
  isSettingsOpen: false,
  isNewCampaignModalOpen: false,

  toggleSidebar: () => set((s) => ({ isSidebarOpen: !s.isSidebarOpen })),
  openSettings: () => set({ isSettingsOpen: true }),
  closeSettings: () => set({ isSettingsOpen: false }),
  openNewCampaignModal: () => set({ isNewCampaignModalOpen: true }),
  closeNewCampaignModal: () => set({ isNewCampaignModalOpen: false }),
}));
