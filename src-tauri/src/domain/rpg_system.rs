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
pub struct EquipmentItem {
    pub name: String,
    pub cost: String,
    pub damage: Option<String>,
    #[serde(default)]
    pub is_bulky: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArcanaItem {
    pub name: String,
    pub effect: String,
    pub cost: Option<String>,
    pub charges: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarterPackage {
    pub name: String,
    pub items: Vec<String>,
    pub gold: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpgSystem {
    pub id: RpgSystemId,
    pub name: String,
    pub system_prompt: String,
    pub character_fields: Vec<CharacterField>,
    #[serde(default)]
    pub mood: Option<RpgSystemMood>,
    #[serde(default)]
    pub opening_hooks: Vec<OpeningHook>,
    #[serde(default)]
    pub rules: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub equipment: Vec<EquipmentItem>,
    #[serde(default)]
    pub arcana: Vec<ArcanaItem>,
    #[serde(default)]
    pub starter_packages: Vec<StarterPackage>,
    #[serde(default)]
    pub setting: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub dice_tables: std::collections::HashMap<String, Vec<String>>,
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
            mood: Some(RpgSystemMood {
                suggested_theme: "fantasy".to_string(),
            }),
            opening_hooks: vec![OpeningHook {
                title: "The Tavern".to_string(),
                description: "You wake up in a tavern.".to_string(),
            }],
            rules: std::collections::HashMap::new(),
            equipment: vec![],
            arcana: vec![],
            starter_packages: vec![],
            setting: std::collections::HashMap::new(),
            dice_tables: std::collections::HashMap::new(),
        };

        let json = serde_json::to_string(&system).unwrap();
        let round_tripped: RpgSystem = serde_json::from_str(&json).unwrap();

        assert_eq!(round_tripped.id.0, "dnd5e");
        assert_eq!(round_tripped.character_fields.len(), 1);
    }

    #[test]
    fn rpg_system_new_fields_default_to_empty() {
        let json = serde_json::json!({
            "id": "test",
            "name": "Test System",
            "system_prompt": "You are a GM.",
            "character_fields": []
        });

        let system: RpgSystem = serde_json::from_value(json).unwrap();
        assert!(system.rules.is_empty());
        assert!(system.equipment.is_empty());
        assert!(system.arcana.is_empty());
        assert!(system.starter_packages.is_empty());
        assert!(system.setting.is_empty());
        assert!(system.dice_tables.is_empty());
    }
}
