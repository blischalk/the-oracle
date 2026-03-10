use serde_json::Value;

use crate::domain::campaign::CampaignState;
use crate::domain::rpg_system::RpgSystem;
use crate::providers::llm_provider::{ToolCall, ToolResult};

pub struct ToolExecutor<'a> {
    pub campaign_state: &'a mut CampaignState,
    pub rpg_system: &'a RpgSystem,
}

impl<'a> ToolExecutor<'a> {
    pub fn execute(&mut self, call: &ToolCall) -> ToolResult {
        let result = match call.tool_name.as_str() {
            "roll_dice" => self.roll_dice(&call.arguments),
            "get_character_sheet" => self.get_character_sheet(),
            "update_character_sheet" => self.update_character_sheet(&call.arguments),
            "save_roll" => self.save_roll(&call.arguments),
            "track_npc" => self.track_npc(&call.arguments),
            "track_story_thread" => self.track_story_thread(&call.arguments),
            "lookup_rules" => self.lookup_rules(&call.arguments),
            "lookup_equipment" => self.lookup_equipment(&call.arguments),
            "lookup_arcana" => self.lookup_arcana(&call.arguments),
            "get_starter_packages" => self.get_starter_packages(),
            "lookup_setting" => self.lookup_setting(&call.arguments),
            "roll_on_table" => self.roll_on_table(&call.arguments),
            unknown => Err(format!("Unknown tool: {unknown}")),
        };

        match result {
            Ok(content) => ToolResult {
                call_id: call.id.clone(),
                tool_name: call.tool_name.clone(),
                content,
                is_error: false,
            },
            Err(msg) => ToolResult {
                call_id: call.id.clone(),
                tool_name: call.tool_name.clone(),
                content: Value::String(msg),
                is_error: true,
            },
        }
    }

    fn roll_dice(&self, args: &Value) -> Result<Value, String> {
        let notation = args["notation"]
            .as_str()
            .ok_or("Missing 'notation' parameter")?;
        let (rolls, total) = parse_and_roll(notation)?;
        Ok(serde_json::json!({
            "notation": notation,
            "rolls": rolls,
            "total": total
        }))
    }

    fn get_character_sheet(&self) -> Result<Value, String> {
        Ok(self.campaign_state.character_data.clone())
    }

    fn update_character_sheet(&mut self, args: &Value) -> Result<Value, String> {
        let updates = args["updates"]
            .as_object()
            .ok_or("Missing 'updates' object")?;

        let valid_fields: std::collections::HashSet<&str> = self
            .rpg_system
            .character_fields
            .iter()
            .map(|f| f.name.as_str())
            .collect();

        let data = match self.campaign_state.character_data.as_object_mut() {
            Some(obj) => obj,
            None => {
                self.campaign_state.character_data = Value::Object(serde_json::Map::new());
                self.campaign_state.character_data.as_object_mut().unwrap()
            }
        };

        let mut accepted = vec![];
        let mut rejected = vec![];

        for (key, value) in updates {
            if valid_fields.contains(key.as_str()) {
                data.insert(key.clone(), value.clone());
                accepted.push(key.as_str());
            } else {
                rejected.push(key.as_str());
            }
        }

        let mut result = serde_json::json!({
            "accepted": accepted,
            "sheet": self.campaign_state.character_data.clone()
        });
        if !rejected.is_empty() {
            result["rejected"] = serde_json::json!(rejected);
            result["note"] = Value::String(
                "Rejected fields are not in this system's schema.".to_string(),
            );
        }
        Ok(result)
    }

    fn track_npc(&mut self, args: &Value) -> Result<Value, String> {
        let name = args["name"].as_str().ok_or("Missing 'name' parameter")?.to_string();
        let description = args["description"]
            .as_str()
            .ok_or("Missing 'description' parameter")?
            .to_string();
        let entry_type = args["type"].as_str().unwrap_or("npc").to_string();
        let status = args["status"].as_str().unwrap_or("active").to_string();

        let data = self.character_data_as_object_mut();
        let npcs = data
            .entry("__npcs")
            .or_insert_with(|| Value::Array(vec![]));
        let list = npcs.as_array_mut().ok_or("__npcs is not an array")?;

        let name_lower = name.to_lowercase();
        let existing = list
            .iter_mut()
            .find(|e| e["name"].as_str().map(|n| n.to_lowercase()) == Some(name_lower.clone()));

        if let Some(entry) = existing {
            entry["description"] = Value::String(description.clone());
            entry["type"] = Value::String(entry_type.clone());
            entry["status"] = Value::String(status.clone());
            Ok(serde_json::json!({ "action": "updated", "name": name, "type": entry_type, "status": status }))
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            list.push(serde_json::json!({
                "id": id,
                "name": name,
                "description": description,
                "type": entry_type,
                "status": status
            }));
            Ok(serde_json::json!({ "action": "added", "name": name, "type": entry_type, "status": status }))
        }
    }

    fn track_story_thread(&mut self, args: &Value) -> Result<Value, String> {
        let title = args["title"].as_str().ok_or("Missing 'title' parameter")?.to_string();
        let description = args["description"]
            .as_str()
            .ok_or("Missing 'description' parameter")?
            .to_string();
        let status = args["status"].as_str().unwrap_or("active").to_string();

        let data = self.character_data_as_object_mut();
        let threads = data
            .entry("__story_threads")
            .or_insert_with(|| Value::Array(vec![]));
        let list = threads
            .as_array_mut()
            .ok_or("__story_threads is not an array")?;

        let title_lower = title.to_lowercase();
        let existing = list
            .iter_mut()
            .find(|e| e["title"].as_str().map(|t| t.to_lowercase()) == Some(title_lower.clone()));

        if let Some(entry) = existing {
            entry["description"] = Value::String(description.clone());
            entry["status"] = Value::String(status.clone());
            Ok(serde_json::json!({ "action": "updated", "title": title, "status": status }))
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            list.push(serde_json::json!({
                "id": id,
                "title": title,
                "description": description,
                "status": status
            }));
            Ok(serde_json::json!({ "action": "added", "title": title, "status": status }))
        }
    }

    fn character_data_as_object_mut(&mut self) -> &mut serde_json::Map<String, Value> {
        if !self.campaign_state.character_data.is_object() {
            self.campaign_state.character_data = Value::Object(serde_json::Map::new());
        }
        self.campaign_state.character_data.as_object_mut().unwrap()
    }

    fn save_roll(&self, args: &Value) -> Result<Value, String> {
        let attribute = args["attribute"]
            .as_str()
            .ok_or("Missing 'attribute' parameter")?;

        let target = self
            .campaign_state
            .character_data
            .get(attribute)
            .and_then(|v| v.as_i64())
            .ok_or_else(|| format!("Attribute '{attribute}' not found on character sheet"))?;

        let roll = roll_d20();
        let success = roll <= target;

        Ok(serde_json::json!({
            "attribute": attribute,
            "target": target,
            "roll": roll,
            "success": success,
            "margin": target - roll
        }))
    }

    fn lookup_rules(&self, args: &Value) -> Result<Value, String> {
        let topic = args["topic"].as_str().ok_or("Missing 'topic' parameter")?;
        let text = self
            .rpg_system
            .rules
            .get(topic)
            .ok_or_else(|| {
                let available: Vec<&str> =
                    self.rpg_system.rules.keys().map(String::as_str).collect();
                format!(
                    "Unknown topic '{topic}'. Available: {}",
                    available.join(", ")
                )
            })?;
        Ok(serde_json::json!({ "topic": topic, "text": text }))
    }

    fn lookup_equipment(&self, args: &Value) -> Result<Value, String> {
        let query = args["query"].as_str().unwrap_or("").to_lowercase();
        let items: Vec<&crate::domain::rpg_system::EquipmentItem> = if query.is_empty() {
            self.rpg_system.equipment.iter().collect()
        } else {
            self.rpg_system
                .equipment
                .iter()
                .filter(|e| e.name.to_lowercase().contains(&query))
                .collect()
        };
        Ok(serde_json::json!({ "items": items }))
    }

    fn lookup_arcana(&self, args: &Value) -> Result<Value, String> {
        let query = args["query"].as_str().unwrap_or("").to_lowercase();
        let items: Vec<&crate::domain::rpg_system::ArcanaItem> = if query.is_empty() {
            self.rpg_system.arcana.iter().collect()
        } else {
            self.rpg_system
                .arcana
                .iter()
                .filter(|a| a.name.to_lowercase().contains(&query))
                .collect()
        };
        Ok(serde_json::json!({ "arcana": items }))
    }

    fn get_starter_packages(&self) -> Result<Value, String> {
        Ok(serde_json::json!({ "packages": self.rpg_system.starter_packages }))
    }

    fn lookup_setting(&self, args: &Value) -> Result<Value, String> {
        let topic = args["topic"].as_str().ok_or("Missing 'topic' parameter")?;
        let text = self
            .rpg_system
            .setting
            .get(topic)
            .ok_or_else(|| {
                let available: Vec<&str> =
                    self.rpg_system.setting.keys().map(String::as_str).collect();
                format!(
                    "Unknown topic '{topic}'. Available: {}",
                    available.join(", ")
                )
            })?;
        Ok(serde_json::json!({ "topic": topic, "text": text }))
    }

    fn roll_on_table(&self, args: &Value) -> Result<Value, String> {
        let table_name = args["table"].as_str().ok_or("Missing 'table' parameter")?;
        let roll = args["roll"]
            .as_i64()
            .ok_or("Missing 'roll' parameter")? as usize;

        let table = self
            .rpg_system
            .dice_tables
            .get(table_name)
            .ok_or_else(|| {
                let available: Vec<&str> = self
                    .rpg_system
                    .dice_tables
                    .keys()
                    .map(String::as_str)
                    .collect();
                format!(
                    "Unknown table '{table_name}'. Available: {}",
                    available.join(", ")
                )
            })?;

        if table.is_empty() {
            return Err(format!("Table '{table_name}' is empty"));
        }

        // 1-indexed, clamp to table length
        let index = (roll.saturating_sub(1)).min(table.len() - 1);
        Ok(serde_json::json!({
            "table": table_name,
            "roll": roll,
            "result": table[index]
        }))
    }
}

// ── Dice engine ──────────────────────────────────────────────────────────────

/// Parses XdY[+/-Z] notation and returns (individual rolls, total).
pub fn parse_and_roll(notation: &str) -> Result<(Vec<i64>, i64), String> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let notation = notation.trim().to_lowercase();

    // Simple seeded PRNG (LCG) — good enough for a game.
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64;

    let mut rng = SimpleRng::new(seed);

    // Parse modifier: look for + or - after the 'd' part
    let (dice_part, modifier) = parse_modifier(&notation)?;

    // Parse XdY
    let parts: Vec<&str> = dice_part.splitn(2, 'd').collect();
    let (count, sides) = match parts.as_slice() {
        [c, s] => {
            let count: u32 = if c.is_empty() {
                1
            } else {
                c.parse()
                    .map_err(|_| format!("Invalid die count: {c}"))?
            };
            let sides: u32 = s.parse().map_err(|_| format!("Invalid die sides: {s}"))?;
            (count, sides)
        }
        _ => return Err(format!("Cannot parse dice notation: {notation}")),
    };

    if sides == 0 {
        return Err("Die sides must be greater than 0".to_string());
    }
    if count == 0 {
        return Ok((vec![], modifier));
    }

    let rolls: Vec<i64> = (0..count)
        .map(|_| (rng.next() % sides as u64 + 1) as i64)
        .collect();
    let total = rolls.iter().sum::<i64>() + modifier;
    Ok((rolls, total))
}

fn parse_modifier(notation: &str) -> Result<(&str, i64), String> {
    // Find the position of + or - that comes AFTER 'd'
    if let Some(d_pos) = notation.find('d') {
        let after_d = &notation[d_pos + 1..];
        if let Some(plus_pos) = after_d.find('+') {
            let modifier: i64 = after_d[plus_pos + 1..]
                .parse()
                .map_err(|_| "Invalid modifier".to_string())?;
            let dice_part = &notation[..d_pos + 1 + plus_pos];
            return Ok((dice_part, modifier));
        }
        if let Some(minus_pos) = after_d.find('-') {
            let modifier: i64 = after_d[minus_pos + 1..]
                .parse()
                .map_err(|_| "Invalid modifier".to_string())?;
            let dice_part = &notation[..d_pos + 1 + minus_pos];
            return Ok((dice_part, -modifier));
        }
    }
    Ok((notation, 0))
}

fn roll_d20() -> i64 {
    let (_, total) = parse_and_roll("d20").unwrap_or((vec![], 10));
    total
}

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0x5DEECE66D,
        }
    }

    fn next(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 33) + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roll_dice_returns_correct_count() {
        let (rolls, _) = parse_and_roll("3d6").unwrap();
        assert_eq!(rolls.len(), 3);
    }

    #[test]
    fn roll_dice_result_in_range() {
        for _ in 0..20 {
            let (rolls, _) = parse_and_roll("d6").unwrap();
            assert!(
                rolls[0] >= 1 && rolls[0] <= 6,
                "roll out of range: {}",
                rolls[0]
            );
        }
    }

    #[test]
    fn roll_dice_with_positive_modifier() {
        let (_, total) = parse_and_roll("2d6+3").unwrap();
        assert!(total >= 5 && total <= 15);
    }

    #[test]
    fn roll_dice_with_negative_modifier() {
        for _ in 0..20 {
            let (_, total) = parse_and_roll("d4-1").unwrap();
            assert!(total >= 0 && total <= 3, "d4-1 out of range: {total}");
        }
    }

    #[test]
    fn single_die_notation() {
        let (rolls, _) = parse_and_roll("d20").unwrap();
        assert_eq!(rolls.len(), 1);
        assert!(rolls[0] >= 1 && rolls[0] <= 20);
    }

    #[test]
    fn invalid_notation_returns_error() {
        assert!(parse_and_roll("banana").is_err());
    }

    #[test]
    fn tool_executor_roll_dice_returns_valid_result() {
        use crate::domain::campaign::CampaignState;
        use crate::domain::rpg_system::{RpgSystemId};

        let mut state = CampaignState::empty_for_campaign("test".to_string());
        let system = RpgSystem {
            id: RpgSystemId("test".to_string()),
            name: "Test".to_string(),
            system_prompt: "".to_string(),
            character_fields: vec![],
            mood: None,
            opening_hooks: vec![],
            rules: std::collections::HashMap::new(),
            equipment: vec![],
            arcana: vec![],
            starter_packages: vec![],
            setting: std::collections::HashMap::new(),
            dice_tables: std::collections::HashMap::new(),
        };

        let mut executor = ToolExecutor {
            campaign_state: &mut state,
            rpg_system: &system,
        };

        let call = ToolCall {
            id: "1".to_string(),
            tool_name: "roll_dice".to_string(),
            arguments: serde_json::json!({ "notation": "2d6" }),
        };

        let result = executor.execute(&call);
        assert!(!result.is_error);
        assert!(result.content["total"].as_i64().is_some());
    }

    #[test]
    fn track_npc_adds_new_entry() {
        use crate::domain::campaign::CampaignState;
        use crate::domain::rpg_system::RpgSystemId;

        let mut state = CampaignState::empty_for_campaign("test".to_string());
        let system = RpgSystem {
            id: RpgSystemId("test".to_string()),
            name: "Test".to_string(),
            system_prompt: "".to_string(),
            character_fields: vec![],
            mood: None,
            opening_hooks: vec![],
            rules: std::collections::HashMap::new(),
            equipment: vec![],
            arcana: vec![],
            starter_packages: vec![],
            setting: std::collections::HashMap::new(),
            dice_tables: std::collections::HashMap::new(),
        };

        let mut executor = ToolExecutor {
            campaign_state: &mut state,
            rpg_system: &system,
        };

        let call = ToolCall {
            id: "1".to_string(),
            tool_name: "track_npc".to_string(),
            arguments: serde_json::json!({
                "name": "Graves",
                "description": "A mysterious contact",
                "type": "npc",
                "status": "active"
            }),
        };

        let result = executor.execute(&call);
        assert!(!result.is_error);
        assert_eq!(result.content["action"], "added");

        let npcs = executor.campaign_state.character_data["__npcs"].as_array().unwrap();
        assert_eq!(npcs.len(), 1);
        assert_eq!(npcs[0]["name"], "Graves");
    }

    #[test]
    fn track_npc_updates_existing_entry() {
        use crate::domain::campaign::CampaignState;
        use crate::domain::rpg_system::RpgSystemId;

        let mut state = CampaignState::empty_for_campaign("test".to_string());
        let system = RpgSystem {
            id: RpgSystemId("test".to_string()),
            name: "Test".to_string(),
            system_prompt: "".to_string(),
            character_fields: vec![],
            mood: None,
            opening_hooks: vec![],
            rules: std::collections::HashMap::new(),
            equipment: vec![],
            arcana: vec![],
            starter_packages: vec![],
            setting: std::collections::HashMap::new(),
            dice_tables: std::collections::HashMap::new(),
        };

        let mut executor = ToolExecutor {
            campaign_state: &mut state,
            rpg_system: &system,
        };

        let add_call = ToolCall {
            id: "1".to_string(),
            tool_name: "track_npc".to_string(),
            arguments: serde_json::json!({
                "name": "Graves",
                "description": "Unknown figure",
                "type": "npc"
            }),
        };
        executor.execute(&add_call);

        let update_call = ToolCall {
            id: "2".to_string(),
            tool_name: "track_npc".to_string(),
            arguments: serde_json::json!({
                "name": "Graves",
                "description": "Revealed as a smuggler",
                "type": "npc",
                "status": "active"
            }),
        };
        let result = executor.execute(&update_call);
        assert_eq!(result.content["action"], "updated");

        let npcs = executor.campaign_state.character_data["__npcs"].as_array().unwrap();
        assert_eq!(npcs.len(), 1);
        assert_eq!(npcs[0]["description"], "Revealed as a smuggler");
    }

    #[test]
    fn track_story_thread_adds_and_updates() {
        use crate::domain::campaign::CampaignState;
        use crate::domain::rpg_system::RpgSystemId;

        let mut state = CampaignState::empty_for_campaign("test".to_string());
        let system = RpgSystem {
            id: RpgSystemId("test".to_string()),
            name: "Test".to_string(),
            system_prompt: "".to_string(),
            character_fields: vec![],
            mood: None,
            opening_hooks: vec![],
            rules: std::collections::HashMap::new(),
            equipment: vec![],
            arcana: vec![],
            starter_packages: vec![],
            setting: std::collections::HashMap::new(),
            dice_tables: std::collections::HashMap::new(),
        };

        let mut executor = ToolExecutor {
            campaign_state: &mut state,
            rpg_system: &system,
        };

        let add_call = ToolCall {
            id: "1".to_string(),
            tool_name: "track_story_thread".to_string(),
            arguments: serde_json::json!({
                "title": "Meet Graves at dawn",
                "description": "Graves wants to meet at the northeast wagon yard on Cinder Street",
                "status": "active"
            }),
        };
        let result = executor.execute(&add_call);
        assert!(!result.is_error);
        assert_eq!(result.content["action"], "added");

        let threads = executor.campaign_state.character_data["__story_threads"].as_array().unwrap();
        assert_eq!(threads.len(), 1);

        let update_call = ToolCall {
            id: "2".to_string(),
            tool_name: "track_story_thread".to_string(),
            arguments: serde_json::json!({
                "title": "Meet Graves at dawn",
                "description": "Graves revealed the wagon yard location — dawn meeting confirmed",
                "status": "active"
            }),
        };
        let result2 = executor.execute(&update_call);
        assert_eq!(result2.content["action"], "updated");

        let threads2 = executor.campaign_state.character_data["__story_threads"].as_array().unwrap();
        assert_eq!(threads2.len(), 1);
    }

    #[test]
    fn tool_executor_unknown_tool_returns_error() {
        use crate::domain::campaign::CampaignState;
        use crate::domain::rpg_system::{RpgSystemId};

        let mut state = CampaignState::empty_for_campaign("test".to_string());
        let system = RpgSystem {
            id: RpgSystemId("test".to_string()),
            name: "Test".to_string(),
            system_prompt: "".to_string(),
            character_fields: vec![],
            mood: None,
            opening_hooks: vec![],
            rules: std::collections::HashMap::new(),
            equipment: vec![],
            arcana: vec![],
            starter_packages: vec![],
            setting: std::collections::HashMap::new(),
            dice_tables: std::collections::HashMap::new(),
        };

        let mut executor = ToolExecutor {
            campaign_state: &mut state,
            rpg_system: &system,
        };

        let call = ToolCall {
            id: "2".to_string(),
            tool_name: "nonexistent_tool".to_string(),
            arguments: serde_json::json!({}),
        };

        let result = executor.execute(&call);
        assert!(result.is_error);
    }
}
