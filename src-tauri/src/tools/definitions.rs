use crate::domain::rpg_system::RpgSystem;
use crate::providers::llm_provider::ToolDefinition;

pub fn build_tool_definitions(system: &RpgSystem) -> Vec<ToolDefinition> {
    let mut tools = universal_tools(system);
    tools.extend(conditional_tools(system));
    tools
}

fn universal_tools(system: &RpgSystem) -> Vec<ToolDefinition> {
    vec![
        roll_dice_tool(),
        get_character_sheet_tool(),
        update_character_sheet_tool(system),
        save_roll_tool(system),
        track_npc_tool(),
        track_story_thread_tool(),
    ]
}

fn track_npc_tool() -> ToolDefinition {
    ToolDefinition {
        name: "track_npc".to_string(),
        description: "Add or update a significant NPC or location in the journal. \
                      Call this whenever a named character or named place becomes relevant to the story. \
                      If an entry with the same name already exists it will be updated, not duplicated. \
                      Use type='npc' for characters and type='location' for places."
            .to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Full name of the NPC or location."
                },
                "description": {
                    "type": "string",
                    "description": "Brief description — role, appearance, or key facts."
                },
                "type": {
                    "type": "string",
                    "enum": ["npc", "location"],
                    "description": "Whether this is a character (npc) or a place (location)."
                },
                "status": {
                    "type": "string",
                    "enum": ["active", "past"],
                    "description": "active = currently relevant; past = no longer in play. Defaults to active."
                }
            },
            "required": ["name", "description", "type"]
        }),
    }
}

fn track_story_thread_tool() -> ToolDefinition {
    ToolDefinition {
        name: "track_story_thread".to_string(),
        description: "Add or update a story thread (quest, mystery, goal, or complication). \
                      Call this whenever a new plot thread opens or its status changes. \
                      If a thread with the same title already exists it will be updated. \
                      Use status='potential' for foreshadowed hooks, 'active' for ongoing threads. \
                      IMPORTANT: whenever the narrative resolves a thread — the goal is achieved, \
                      the mystery is solved, the enemy is defeated, the complication ends — you MUST \
                      call this tool with status='completed' in that same response. Do not wait for \
                      the player to mark it manually."
            .to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "description": "Short title for the thread (e.g. 'Find the missing merchant')."
                },
                "description": {
                    "type": "string",
                    "description": "Current summary of the thread — what is known and what is at stake."
                },
                "status": {
                    "type": "string",
                    "enum": ["active", "potential", "completed"],
                    "description": "active = in progress; potential = hinted at; completed = resolved."
                }
            },
            "required": ["title", "description", "status"]
        }),
    }
}

fn conditional_tools(system: &RpgSystem) -> Vec<ToolDefinition> {
    let mut tools = vec![];
    if !system.rules.is_empty() {
        tools.push(lookup_rules_tool(system));
    }
    if !system.equipment.is_empty() {
        tools.push(lookup_equipment_tool());
    }
    if !system.arcana.is_empty() {
        tools.push(lookup_arcana_tool());
    }
    if !system.starter_packages.is_empty() {
        tools.push(get_starter_packages_tool());
    }
    if !system.setting.is_empty() {
        tools.push(lookup_setting_tool(system));
    }
    if !system.dice_tables.is_empty() {
        tools.push(roll_on_table_tool(system));
    }
    tools
}

fn roll_dice_tool() -> ToolDefinition {
    ToolDefinition {
        name: "roll_dice".to_string(),
        description: "Roll dice using standard RPG notation (e.g. '3d6', 'd20', '2d6+3'). \
                      Always call this before narrating any dice result — never invent a number."
            .to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "notation": {
                    "type": "string",
                    "description": "Dice notation such as '3d6', 'd20', '2d6+3', 'd4-1'."
                }
            },
            "required": ["notation"]
        }),
    }
}

fn get_character_sheet_tool() -> ToolDefinition {
    ToolDefinition {
        name: "get_character_sheet".to_string(),
        description: "Read the current authoritative character sheet. Always call this before \
                      referencing any stat value in narration."
            .to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
    }
}

fn update_character_sheet_tool(system: &RpgSystem) -> ToolDefinition {
    let field_list: Vec<String> = system
        .character_fields
        .iter()
        .map(|f| format!("\"{}\" ({})", f.name, f.label))
        .collect();
    let field_summary = if field_list.is_empty() {
        "No fields defined.".to_string()
    } else {
        field_list.join(", ")
    };

    ToolDefinition {
        name: "update_character_sheet".to_string(),
        description: format!(
            "Write one or more character sheet fields. Call this immediately after ANY change \
             to a tracked value. This includes: \
             ITEM ACQUISITION — whenever the narrative has the character pick up, find, receive, \
             loot, steal, or otherwise acquire any item, you MUST append it to the relevant \
             inventory/equipment field in that same response (do not just narrate the pickup \
             without updating the sheet); \
             ITEM LOSS — whenever an item is dropped, consumed, sold, destroyed, or taken; \
             CURRENCY TRANSACTIONS — any time the character spends, earns, or loses money the \
             updated amount MUST be written here in the same response; \
             STAT CHANGES — damage taken, healing received, HP changes; \
             CHARACTER CREATION — set all initial values when rolling the character. \
             FORMATTING — when writing list-style fields (inventory, equipment, items), \
             each entry must be a single line. Use commas to separate an item's properties, \
             never a newline. Correct: 'Musket (d10, bulky)'. \
             Wrong: 'Musket (d10\\nbulky)' — that creates broken split list items. \
             You MUST use the exact field names listed here — wrong names are silently rejected. \
             Valid fields for this system: {field_summary}."
        ),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "updates": {
                    "type": "object",
                    "description": "Map of exact field_name to new_value. Use only the field names listed in the tool description.",
                    "additionalProperties": true
                }
            },
            "required": ["updates"]
        }),
    }
}

fn save_roll_tool(system: &RpgSystem) -> ToolDefinition {
    let numeric_fields: Vec<&str> = system
        .character_fields
        .iter()
        .filter(|f| matches!(f.field_type, crate::domain::rpg_system::FieldType::Number))
        .map(|f| f.name.as_str())
        .collect();
    let examples = if numeric_fields.is_empty() {
        "'str', 'dex', 'wil'".to_string()
    } else {
        numeric_fields
            .iter()
            .take(3)
            .map(|n| format!("'{n}'"))
            .collect::<Vec<_>>()
            .join(", ")
    };

    ToolDefinition {
        name: "save_roll".to_string(),
        description: "Roll d20 for a saving throw against a character attribute. \
                      Returns the roll, the target, and whether it succeeded."
            .to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "attribute": {
                    "type": "string",
                    "description": format!("Exact field name of the numeric attribute to save against (e.g. {examples}).")
                }
            },
            "required": ["attribute"]
        }),
    }
}

fn lookup_rules_tool(system: &RpgSystem) -> ToolDefinition {
    let topics: Vec<String> = system.rules.keys().cloned().collect();
    ToolDefinition {
        name: "lookup_rules".to_string(),
        description: format!(
            "Look up an exact rule. Available topics: {}.",
            topics.join(", ")
        ),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "topic": {
                    "type": "string",
                    "description": format!("One of: {}", topics.join(", "))
                }
            },
            "required": ["topic"]
        }),
    }
}

fn lookup_equipment_tool() -> ToolDefinition {
    ToolDefinition {
        name: "lookup_equipment".to_string(),
        description: "Find equipment items by name or browse the full list.".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Optional partial name to search for. Omit to get all items."
                }
            },
            "required": []
        }),
    }
}

fn lookup_arcana_tool() -> ToolDefinition {
    ToolDefinition {
        name: "lookup_arcana".to_string(),
        description: "Find arcana or magic items by name, or browse the full list.".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Optional partial name to search for. Omit to get all arcana."
                }
            },
            "required": []
        }),
    }
}

fn get_starter_packages_tool() -> ToolDefinition {
    ToolDefinition {
        name: "get_starter_packages".to_string(),
        description: "Get all available starting equipment packages for character creation."
            .to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {},
            "required": []
        }),
    }
}

fn lookup_setting_tool(system: &RpgSystem) -> ToolDefinition {
    let topics: Vec<String> = system.setting.keys().cloned().collect();
    ToolDefinition {
        name: "lookup_setting".to_string(),
        description: format!(
            "Look up world and lore information. Available topics: {}.",
            topics.join(", ")
        ),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "topic": {
                    "type": "string",
                    "description": format!("One of: {}", topics.join(", "))
                }
            },
            "required": ["topic"]
        }),
    }
}

fn roll_on_table_tool(system: &RpgSystem) -> ToolDefinition {
    let tables: Vec<String> = system.dice_tables.keys().cloned().collect();
    ToolDefinition {
        name: "roll_on_table".to_string(),
        description: format!(
            "Roll on a random table. First call roll_dice to get a number, then pass it here. \
             Available tables: {}.",
            tables.join(", ")
        ),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "table": {
                    "type": "string",
                    "description": format!("One of: {}", tables.join(", "))
                },
                "roll": {
                    "type": "integer",
                    "description": "The dice result (1-indexed)."
                }
            },
            "required": ["table", "roll"]
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::rpg_system::RpgSystemId;

    fn minimal_system() -> RpgSystem {
        RpgSystem {
            id: RpgSystemId("test".to_string()),
            name: "Test".to_string(),
            system_prompt: "You are a GM.".to_string(),
            character_fields: vec![],
            mood: None,
            opening_hooks: vec![],
            rules: std::collections::HashMap::new(),
            equipment: vec![],
            arcana: vec![],
            starter_packages: vec![],
            setting: std::collections::HashMap::new(),
            dice_tables: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn universal_tools_always_present() {
        let system = minimal_system();
        let tools = build_tool_definitions(&system);
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"roll_dice"));
        assert!(names.contains(&"get_character_sheet"));
        assert!(names.contains(&"update_character_sheet"));
        assert!(names.contains(&"save_roll"));
        assert!(names.contains(&"track_npc"));
        assert!(names.contains(&"track_story_thread"));
    }

    #[test]
    fn conditional_tools_absent_when_collections_empty() {
        let system = minimal_system();
        let tools = build_tool_definitions(&system);
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(!names.contains(&"lookup_rules"));
        assert!(!names.contains(&"lookup_equipment"));
        assert!(!names.contains(&"lookup_arcana"));
        assert!(!names.contains(&"get_starter_packages"));
        assert!(!names.contains(&"lookup_setting"));
        assert!(!names.contains(&"roll_on_table"));
    }

    #[test]
    fn conditional_tools_present_when_collections_populated() {
        let mut system = minimal_system();
        system.rules.insert("combat".to_string(), "Fight!".to_string());
        system.equipment.push(crate::domain::rpg_system::EquipmentItem {
            name: "Sword".to_string(),
            cost: "5g".to_string(),
            damage: Some("d6".to_string()),
            is_bulky: false,
            notes: None,
        });
        system.arcana.push(crate::domain::rpg_system::ArcanaItem {
            name: "Fireball".to_string(),
            effect: "Burns things".to_string(),
            cost: None,
            charges: Some(3),
        });
        system.starter_packages.push(crate::domain::rpg_system::StarterPackage {
            name: "Warrior".to_string(),
            items: vec!["Sword".to_string()],
            gold: Some("5g".to_string()),
            notes: None,
        });
        system.setting.insert("world".to_string(), "A dark place".to_string());
        system.dice_tables.insert("encounters".to_string(), vec!["Goblin".to_string()]);

        let tools = build_tool_definitions(&system);
        let names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"lookup_rules"));
        assert!(names.contains(&"lookup_equipment"));
        assert!(names.contains(&"lookup_arcana"));
        assert!(names.contains(&"get_starter_packages"));
        assert!(names.contains(&"lookup_setting"));
        assert!(names.contains(&"roll_on_table"));
    }
}
