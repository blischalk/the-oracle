import { create } from "zustand";

interface UiStore {
  isSidebarOpen: boolean;
  isSettingsOpen: boolean;
  isNewCampaignModalOpen: boolean;
  searchQuery: string;

  toggleSidebar: () => void;
  openSettings: () => void;
  closeSettings: () => void;
  openNewCampaignModal: () => void;
  closeNewCampaignModal: () => void;
  setSearchQuery: (q: string) => void;
}

export const useUiStore = create<UiStore>((set) => ({
  isSidebarOpen: true,
  isSettingsOpen: false,
  isNewCampaignModalOpen: false,
  searchQuery: "",

  toggleSidebar: () => set((s) => ({ isSidebarOpen: !s.isSidebarOpen })),
  openSettings: () => set({ isSettingsOpen: true }),
  closeSettings: () => set({ isSettingsOpen: false }),
  openNewCampaignModal: () => set({ isNewCampaignModalOpen: true }),
  closeNewCampaignModal: () => set({ isNewCampaignModalOpen: false }),
  setSearchQuery: (q: string) => set({ searchQuery: q }),
}));
