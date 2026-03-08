use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RpgSystemId(pub String);

impl std::fmt::Display for RpgSystemId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FieldType {
    Text,
    Number,
    Boolean,
    Select { options: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterField {
    pub name: String,
    pub field_type: FieldType,
    pub label: String,
    pub default_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpeningHook {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpgSystemMood {
    pub suggested_theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpgSystem {
    pub id: RpgSystemId,
    pub name: String,
    pub system_prompt: String,
    pub character_fields: Vec<CharacterField>,
    pub mood: RpgSystemMood,
    pub opening_hooks: Vec<OpeningHook>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpg_system_id_displays_its_inner_value() {
        let id = RpgSystemId("dnd5e".to_string());
        assert_eq!(id.to_string(), "dnd5e");
    }

    #[test]
    fn rpg_system_serialises_and_deserialises() {
        let system = RpgSystem {
            id: RpgSystemId("dnd5e".to_string()),
            name: "Dungeons & Dragons 5e".to_string(),
            system_prompt: "You are a dungeon master.".to_string(),
            character_fields: vec![CharacterField {
                name: "strength".to_string(),
                field_type: FieldType::Number,
                label: "Strength".to_string(),
                default_value: Some(serde_json::json!(10)),
            }],
            mood: RpgSystemMood {
                suggested_theme: "fantasy".to_string(),
            },
            opening_hooks: vec![OpeningHook {
                title: "The Tavern".to_string(),
                description: "You wake up in a tavern.".to_string(),
            }],
        };

        let json = serde_json::to_string(&system).unwrap();
        let round_tripped: RpgSystem = serde_json::from_str(&json).unwrap();

        assert_eq!(round_tripped.id.0, "dnd5e");
        assert_eq!(round_tripped.character_fields.len(), 1);
    }
}
