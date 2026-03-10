import { useEffect } from "react";
import { useSettingsStore } from "../stores/settingsStore";
import { THEMES, DEFAULT_THEME_ID } from "../themes";

export function useTheme() {
  const { settings, updateSettings } = useSettingsStore();

  useEffect(() => {
    const html = document.documentElement;
    THEMES.forEach((t) => html.classList.remove(t.cssClass));
    const active =
      THEMES.find((t) => t.id === settings.theme) ??
      THEMES.find((t) => t.id === DEFAULT_THEME_ID) ??
      THEMES[0];
    html.classList.add(active.cssClass);
  }, [settings.theme]);

  function setTheme(themeId: string) {
    updateSettings({ theme: themeId });
  }

  return { activeTheme: settings.theme, setTheme, themes: THEMES };
}
