import { useEffect } from "react";
import { useSettingsStore } from "../stores/settingsStore";

export function useSettings() {
  const store = useSettingsStore();

  useEffect(() => {
    if (!store.isLoaded) {
      store.loadSettings();
      store.loadProviders();
    }
  }, [store.isLoaded]);

  return store;
}
