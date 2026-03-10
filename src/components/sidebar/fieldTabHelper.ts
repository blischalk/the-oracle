export type FieldTab = "character" | "inventory" | "skills";

const INVENTORY_KEYWORDS = [
  "inventory",
  "item",
  "equipment",
  "weapon",
  "armor",
  "armour",
  "coin",
  "gold",
  "silver",
  "ration",
  "torch",
  "pouch",
  "backpack",
];

const SKILLS_KEYWORDS = [
  "skill",
  "ability",
  "spell",
  "magic",
  "feat",
  "talent",
  "power",
  "mutation",
  "gift",
];

export function getFieldTab(fieldName: string): FieldTab {
  const lower = fieldName.toLowerCase();

  if (INVENTORY_KEYWORDS.some((kw) => lower.includes(kw))) {
    return "inventory";
  }

  if (SKILLS_KEYWORDS.some((kw) => lower.includes(kw))) {
    return "skills";
  }

  return "character";
}
