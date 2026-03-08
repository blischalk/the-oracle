import { describe, it, expect, beforeEach } from "vitest";

const { useUiStore } = await import("../../stores/uiStore");

describe("uiStore", () => {
  beforeEach(() => {
    useUiStore.setState({
      isSidebarOpen: true,
      isSettingsOpen: false,
      isNewCampaignModalOpen: false,
      searchQuery: "",
    });
  });

  it("toggles sidebar", () => {
    useUiStore.getState().toggleSidebar();
    expect(useUiStore.getState().isSidebarOpen).toBe(false);
    useUiStore.getState().toggleSidebar();
    expect(useUiStore.getState().isSidebarOpen).toBe(true);
  });

  it("opens and closes settings", () => {
    useUiStore.getState().openSettings();
    expect(useUiStore.getState().isSettingsOpen).toBe(true);
    useUiStore.getState().closeSettings();
    expect(useUiStore.getState().isSettingsOpen).toBe(false);
  });

  it("opens and closes new campaign modal", () => {
    useUiStore.getState().openNewCampaignModal();
    expect(useUiStore.getState().isNewCampaignModalOpen).toBe(true);
    useUiStore.getState().closeNewCampaignModal();
    expect(useUiStore.getState().isNewCampaignModalOpen).toBe(false);
  });

  it("sets search query", () => {
    useUiStore.getState().setSearchQuery("dungeon");
    expect(useUiStore.getState().searchQuery).toBe("dungeon");
    useUiStore.getState().setSearchQuery("");
    expect(useUiStore.getState().searchQuery).toBe("");
  });
});
