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
];

export const DEFAULT_THEME_ID = "ink";
