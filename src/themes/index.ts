export interface Theme {
  id: string;
  label: string;
  cssClass: string;
}

export const THEMES: Theme[] = [
  { id: "ink",       label: "Ink",       cssClass: "theme-ink" },
  { id: "parchment", label: "Parchment", cssClass: "theme-parchment" },
  { id: "ember",     label: "Ember",     cssClass: "theme-ember" },
  { id: "terminal",  label: "Terminal",  cssClass: "theme-terminal" },
  { id: "void",      label: "Void",      cssClass: "theme-void" },
  { id: "slate",     label: "Slate",     cssClass: "theme-slate" },
  { id: "ashwood",   label: "Ashwood",   cssClass: "theme-ashwood" },
  { id: "crimson",   label: "Crimson",   cssClass: "theme-crimson" },
  { id: "seafarer",  label: "Seafarer",  cssClass: "theme-seafarer" },
  { id: "dusk",      label: "Dusk",      cssClass: "theme-dusk" },
  { id: "ironwood",  label: "Ironwood",  cssClass: "theme-ironwood" },
  { id: "silver",    label: "Silver",    cssClass: "theme-silver" },
];

export const DEFAULT_THEME_ID = "ink";
