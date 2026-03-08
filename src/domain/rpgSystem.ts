export interface OpeningHook {
  title: string;
  description: string;
}

export type FieldType =
  | { type: "Text" }
  | { type: "Number" }
  | { type: "Boolean" }
  | { type: "Select"; options: string[] };

export interface CharacterField {
  name: string;
  field_type: FieldType;
  label: string;
  default_value?: unknown;
}

export interface RpgSystem {
  id: string;
  name: string;
  character_fields: CharacterField[];
  opening_hooks: OpeningHook[];
  mood: { suggested_theme: string };
}
