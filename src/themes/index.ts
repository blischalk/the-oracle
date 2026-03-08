export interface Theme {
  id: string;
  label: string;
  cssClass: string;
  suggestedFor: string[];
}

export const THEMES: Theme[] = [
  { id: "default", label: "Default", cssClass: "theme-default", suggestedFor: [] },
  { id: "dungeon", label: "Dungeon", cssClass: "theme-dungeon", suggestedFor: ["old-school-essentials", "cairn"] },
  { id: "mork-borg", label: "Mörk Borg", cssClass: "theme-mork-borg", suggestedFor: ["mork-borg"] },
  { id: "ultraviolet", label: "Ultraviolet", cssClass: "theme-ultraviolet", suggestedFor: ["ultraviolet-grasslands"] },
  { id: "electric", label: "Electric", cssClass: "theme-electric", suggestedFor: ["electric-bastionland", "into-the-odd"] },
  { id: "cosmos", label: "Cosmos", cssClass: "theme-cosmos", suggestedFor: ["between-the-skies", "troika", "runecairn"] },
];

export function suggestedThemeForSystem(rpgSystemId: string): string {
  const match = THEMES.find((t) => t.suggestedFor.includes(rpgSystemId));
  return match?.id ?? "default";
}
