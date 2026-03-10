import { create } from "zustand";

export type SidebarTab =
  | "campaigns"
  | "character"
  | "inventory"
  | "skills"
  | "journal"
  | "quests";

interface UiStore {
  isSidebarOpen: boolean;
  isSettingsOpen: boolean;
  isNewCampaignModalOpen: boolean;
  searchQuery: string;
  activeSidebarTab: SidebarTab;

  toggleSidebar: () => void;
  openSettings: () => void;
  closeSettings: () => void;
  openNewCampaignModal: () => void;
  closeNewCampaignModal: () => void;
  setSearchQuery: (q: string) => void;
  setActiveSidebarTab: (tab: SidebarTab) => void;
}

export const useUiStore = create<UiStore>((set) => ({
  isSidebarOpen: true,
  isSettingsOpen: false,
  isNewCampaignModalOpen: false,
  searchQuery: "",
  activeSidebarTab: "campaigns",

  toggleSidebar: () => set((s) => ({ isSidebarOpen: !s.isSidebarOpen })),
  openSettings: () => set({ isSettingsOpen: true }),
  closeSettings: () => set({ isSettingsOpen: false }),
  openNewCampaignModal: () => set({ isNewCampaignModalOpen: true }),
  closeNewCampaignModal: () => set({ isNewCampaignModalOpen: false }),
  setSearchQuery: (q: string) => set({ searchQuery: q }),
  setActiveSidebarTab: (tab: SidebarTab) => set({ activeSidebarTab: tab }),
}));
