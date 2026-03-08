import { useEffect } from "react";
import { useCampaignStore } from "../stores/campaignStore";

export function useCampaigns() {
  useEffect(() => {
    // Use getState() so we always call the current loadCampaigns (avoids stale closure)
    useCampaignStore.getState().loadCampaigns();
  }, []);

  return useCampaignStore();
}
