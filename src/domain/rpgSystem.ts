export interface OpeningHook {
  title: string;
  description: string;
}

export interface CharacterField {
  name: string;
  field_type: string;
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
