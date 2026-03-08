import { useEffect } from "react";
import { useCampaignStore } from "../stores/campaignStore";

export function useCampaigns() {
  const store = useCampaignStore();

  useEffect(() => {
    store.loadCampaigns();
  }, []);

  return store;
}
