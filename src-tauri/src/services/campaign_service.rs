use std::sync::Arc;

use crate::domain::campaign::{Campaign, CampaignState, Message};
use crate::domain::rpg_system::RpgSystem;
use crate::persistence::campaign_repository::{CampaignRepository, MessageRepository};
use crate::providers::llm_provider::ChatMessage;
use crate::services::prompt_library::PromptLibrary;
use crate::services::rpg_system_registry::RpgSystemRegistry;

const MAX_CONTEXT_MESSAGES: usize = 15;

pub enum GreetingKind {
    NewCampaign,
    ResumeCampaign,
}

pub struct CampaignService {
    campaign_repository: Arc<CampaignRepository>,
    message_repository: Arc<MessageRepository>,
    rpg_registry: Arc<RpgSystemRegistry>,
    prompt_library: Arc<PromptLibrary>,
}

impl CampaignService {
    pub fn new(
        campaign_repository: Arc<CampaignRepository>,
        message_repository: Arc<MessageRepository>,
        rpg_registry: Arc<RpgSystemRegistry>,
        prompt_library: Arc<PromptLibrary>,
    ) -> Self {
        Self {
            campaign_repository,
            message_repository,
            rpg_registry,
            prompt_library,
        }
    }

    pub fn create_campaign(&self, name: &str, rpg_system_id: &str) -> anyhow::Result<Campaign> {
        let campaign = Campaign::created_now(name.to_string(), rpg_system_id.to_string());
        self.campaign_repository.create(&campaign)?;
        Ok(campaign)
    }

    pub fn list_campaigns(&self) -> anyhow::Result<Vec<Campaign>> {
        self.campaign_repository.list_active()
    }

    pub fn get_campaign(&self, id: &str) -> anyhow::Result<Option<Campaign>> {
        self.campaign_repository.find_by_id(id)
    }

    pub fn archive_campaign(&self, id: &str) -> anyhow::Result<()> {
        self.campaign_repository.archive(id)
    }

    /// Permanently deletes a campaign and all its messages and state.
    pub fn delete_campaign(&self, id: &str) -> anyhow::Result<()> {
        self.campaign_repository.delete(id)
    }

    pub fn update_campaign_name(&self, id: &str, name: &str) -> anyhow::Result<()> {
        self.campaign_repository.update_name(id, name)
    }

    pub fn save_message(&self, message: &Message) -> anyhow::Result<()> {
        self.message_repository.save(message)
    }

    pub fn get_messages(&self, campaign_id: &str) -> anyhow::Result<Vec<Message>> {
        self.message_repository.find_by_campaign(campaign_id)
    }

    pub fn get_messages_recent(
        &self,
        campaign_id: &str,
        limit: usize,
    ) -> anyhow::Result<(Vec<Message>, bool)> {
        self.message_repository.find_recent(campaign_id, limit)
    }

    pub fn get_messages_before(
        &self,
        campaign_id: &str,
        before_created_at: &str,
        limit: usize,
    ) -> anyhow::Result<(Vec<Message>, bool)> {
        self.message_repository
            .find_before(campaign_id, before_created_at, limit)
    }

    pub fn get_campaign_state(&self, campaign_id: &str) -> anyhow::Result<CampaignState> {
        self.campaign_repository
            .find_state(campaign_id)
            .map(|opt| opt.unwrap_or_else(|| CampaignState::empty_for_campaign(campaign_id.to_string())))
    }

    pub fn save_campaign_state(&self, state: &CampaignState) -> anyhow::Result<()> {
        self.campaign_repository.save_state(state)
    }

    pub fn patch_character_data(
        &self,
        campaign_id: &str,
        patch: serde_json::Value,
    ) -> anyhow::Result<CampaignState> {
        let mut state = self.get_campaign_state(campaign_id)?;
        if let (serde_json::Value::Object(data), serde_json::Value::Object(patch_map)) =
            (&mut state.character_data, patch)
        {
            for (key, value) in patch_map {
                data.insert(key, value);
            }
        }
        state.updated_at = chrono::Utc::now();
        self.save_campaign_state(&state)?;
        Ok(state)
    }

    pub fn build_llm_context(
        &self,
        campaign_id: &str,
        rpg_system: &RpgSystem,
        campaign_state: &CampaignState,
    ) -> anyhow::Result<Vec<ChatMessage>> {
        let all_messages = self.message_repository.find_by_campaign(campaign_id)?;
        let recent_messages = take_last_n_messages(all_messages, MAX_CONTEXT_MESSAGES);

        let mut context = vec![system_message_for(&self.prompt_library, rpg_system, campaign_state)];
        context.extend(
            recent_messages
                .into_iter()
                .map(domain_message_to_chat_message),
        );

        if stats_not_yet_assigned(campaign_state, rpg_system) {
            append_creation_reminder(&mut context, &self.prompt_library);
        }

        Ok(context)
    }

    /// Builds context for a GM greeting: new campaign (character creation) or resume (recap + next decision).
    pub fn build_greeting_context(
        &self,
        campaign_id: &str,
        rpg_system: &RpgSystem,
        campaign_state: &CampaignState,
        kind: GreetingKind,
    ) -> anyhow::Result<Vec<ChatMessage>> {
        let mut context = vec![system_message_for(&self.prompt_library, rpg_system, campaign_state)];

        let instruction = match kind {
            GreetingKind::NewCampaign => {
                let stat_clause = build_stat_clause(rpg_system);
                self.prompt_library
                    .render("tasks/greeting_new_campaign", &[("stat_clause", &stat_clause)])
                    .unwrap_or_default()
            }
            GreetingKind::ResumeCampaign => self
                .prompt_library
                .get("tasks/greeting_resume_campaign")
                .unwrap_or_default()
                .to_string(),
        };

        match kind {
            GreetingKind::NewCampaign => {
                context.push(ChatMessage::user(instruction));
            }
            GreetingKind::ResumeCampaign => {
                let all_messages = self.message_repository.find_by_campaign(campaign_id)?;
                let recent = take_last_n_messages(all_messages, MAX_CONTEXT_MESSAGES);
                context.extend(recent.into_iter().map(domain_message_to_chat_message));
                context.push(ChatMessage::user(instruction));
            }
        }

        Ok(context)
    }

    pub fn prompt_library(&self) -> &PromptLibrary {
        &self.prompt_library
    }

    pub fn rpg_registry(&self) -> &RpgSystemRegistry {
        &self.rpg_registry
    }
}

/// Appends a creation reminder to the last user message in the context.
/// This places the instruction at the highest-attention position (end of context,
/// in the user turn) so it is not overridden by conversational momentum.
fn append_creation_reminder(context: &mut Vec<ChatMessage>, prompt_library: &PromptLibrary) {
    let reminder = prompt_library
        .get("system/creation_reminder")
        .unwrap_or_default();
    if reminder.is_empty() {
        return;
    }
    if let Some(last_msg) = context.last_mut() {
        if last_msg.role == "user" {
            last_msg.content = format!("{}\n\n{reminder}", last_msg.content);
        }
    }
}

fn build_stat_clause(rpg_system: &RpgSystem) -> String {
    let stat_labels = numeric_stat_labels(rpg_system);
    if stat_labels.is_empty() {
        return String::new();
    }
    format!(
        " The stats tracked for this system are: {}. \
         As soon as the player gives their name (and background if applicable), \
         assign or roll ALL of these stats in that SAME response — do not wait. \
         State each value explicitly as a number (e.g. '{} 3, {} 4').",
        stat_labels.join(", "),
        stat_labels.first().cloned().unwrap_or_default(),
        stat_labels.get(1).cloned().unwrap_or_default(),
    )
}

fn numeric_stat_labels(rpg_system: &RpgSystem) -> Vec<String> {
    rpg_system
        .character_fields
        .iter()
        .filter(|f| matches!(f.field_type, crate::domain::rpg_system::FieldType::Number))
        .map(|f| f.label.clone())
        .collect()
}

fn system_message_for(
    prompt_library: &PromptLibrary,
    rpg_system: &RpgSystem,
    campaign_state: &CampaignState,
) -> ChatMessage {
    let role = prompt_library.get("system/role").unwrap_or_default();
    let forbidden = prompt_library.get("system/forbidden").unwrap_or_default();
    let formatting = prompt_library.get("system/formatting").unwrap_or_default();
    let pacing = prompt_library.get("system/pacing").unwrap_or_default();
    let continuity = prompt_library.get("system/continuity").unwrap_or_default();
    let mechanics = prompt_library.get("system/mechanics").unwrap_or_default();
    let character_creation = prompt_library.get("system/character_creation").unwrap_or_default();
    let confrontation = prompt_library.get("system/confrontation").unwrap_or_default();
    let naming = prompt_library.get("system/naming").unwrap_or_default();

    let world_state = build_world_state_section(campaign_state);
    let world_state_block = if world_state.is_empty() {
        String::new()
    } else {
        format!("\n\n{world_state}")
    };

    let prompt = if stats_not_yet_assigned(campaign_state, rpg_system) {
        // creation_alert goes LAST — after the RPG system YAML — so it wins
        // the recency competition against the YAML's "open a compelling scene" instruction.
        let creation_alert = prompt_library.get("system/creation_alert").unwrap_or_default();
        format!(
            "{role}\n\n{forbidden}\n\n{formatting}\n\n{pacing}\n\n{continuity}\n\n\
             {mechanics}\n\n{character_creation}\n\n{confrontation}\n\n\
             {naming}{world_state_block}\n\n{}\n\n{creation_alert}",
            rpg_system.system_prompt
        )
    } else {
        format!(
            "{role}\n\n{forbidden}\n\n{formatting}\n\n{pacing}\n\n{continuity}\n\n\
             {mechanics}\n\n{character_creation}\n\n{confrontation}\n\n\
             {naming}{world_state_block}\n\n{}",
            rpg_system.system_prompt
        )
    };

    ChatMessage::system(prompt)
}

/// Returns true when the RPG system has numeric stats but none have been written
/// to the character sheet yet — i.e., the player has given their name but the LLM
/// has not yet called update_character_sheet with real values.
fn stats_not_yet_assigned(state: &CampaignState, rpg_system: &RpgSystem) -> bool {
    let numeric_fields: Vec<&str> = rpg_system
        .character_fields
        .iter()
        .filter(|f| matches!(f.field_type, crate::domain::rpg_system::FieldType::Number))
        .map(|f| f.name.as_str())
        .collect();

    if numeric_fields.is_empty() {
        return false;
    }

    let Some(data) = state.character_data.as_object() else {
        return true;
    };

    !numeric_fields
        .iter()
        .any(|name| data.contains_key(*name))
}

fn build_world_state_section(state: &CampaignState) -> String {
    let mut sections: Vec<String> = Vec::new();

    if let Some(npcs) = state.character_data.get("__npcs").and_then(|v| v.as_array()) {
        if !npcs.is_empty() {
            let lines: Vec<String> = npcs
                .iter()
                .filter_map(|entry| {
                    let name = entry["name"].as_str()?;
                    let desc = entry["description"].as_str().unwrap_or("");
                    let status = entry["status"].as_str().unwrap_or("active");
                    Some(format!("- {name} ({status}): {desc}"))
                })
                .collect();
            if !lines.is_empty() {
                sections.push(format!("KNOWN NPCs AND LOCATIONS:\n{}", lines.join("\n")));
            }
        }
    }

    if let Some(threads) = state
        .character_data
        .get("__story_threads")
        .and_then(|v| v.as_array())
    {
        let open_lines: Vec<String> = threads
            .iter()
            .filter(|t| t["status"].as_str() != Some("completed"))
            .filter_map(|t| {
                let title = t["title"].as_str()?;
                let desc = t["description"].as_str().unwrap_or("");
                let status = t["status"].as_str().unwrap_or("active");
                Some(format!("- [{status}] {title}: {desc}"))
            })
            .collect();
        if !open_lines.is_empty() {
            sections.push(format!("OPEN STORY THREADS:\n{}", open_lines.join("\n")));
        }
    }

    if sections.is_empty() {
        return String::new();
    }

    format!(
        "WORLD STATE — treat this as your authoritative record. \
         Never contradict names, genders, roles, or facts listed here, \
         even if the conversation history is ambiguous:\n\n{}",
        sections.join("\n\n")
    )
}

fn take_last_n_messages(messages: Vec<Message>, count: usize) -> Vec<Message> {
    if messages.len() <= count {
        messages
    } else {
        messages
            .into_iter()
            .rev()
            .take(count)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }
}

fn domain_message_to_chat_message(message: Message) -> ChatMessage {
    ChatMessage {
        role: message.role.to_string(),
        content: message.content,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::campaign::MessageRole;
    use crate::persistence::database::Database;

    fn build_service() -> CampaignService {
        let database = Database::open_in_memory().unwrap();
        let campaign_repo = Arc::new(CampaignRepository::new(database.connection.clone()));
        let message_repo = Arc::new(MessageRepository::new(database.connection.clone()));
        let registry =
            Arc::new(RpgSystemRegistry::load(std::path::Path::new("/nonexistent")).unwrap());
        let prompt_library = Arc::new(PromptLibrary::empty());

        CampaignService::new(campaign_repo, message_repo, registry, prompt_library)
    }

    #[test]
    fn create_campaign_persists_and_returns_campaign() {
        let service = build_service();
        let campaign = service.create_campaign("Quest", "dnd5e").unwrap();

        assert_eq!(campaign.name, "Quest");
        assert_eq!(campaign.rpg_system_id, "dnd5e");

        let found = service.get_campaign(&campaign.id.0).unwrap().unwrap();
        assert_eq!(found.name, "Quest");
    }

    #[test]
    fn list_campaigns_only_returns_active_campaigns() {
        let service = build_service();
        let active = service.create_campaign("Active", "dnd5e").unwrap();
        let to_archive = service.create_campaign("Archived", "dnd5e").unwrap();

        service.archive_campaign(&to_archive.id.0).unwrap();

        let active_list = service.list_campaigns().unwrap();
        assert_eq!(active_list.len(), 1);
        assert_eq!(active_list[0].id.0, active.id.0);
    }

    #[test]
    fn save_and_get_messages_round_trips() {
        let service = build_service();
        let campaign = service.create_campaign("Campaign", "dnd5e").unwrap();
        let message = Message::new(
            campaign.id.0.clone(),
            MessageRole::User,
            "I enter the dungeon.".to_string(),
        );

        service.save_message(&message).unwrap();

        let messages = service.get_messages(&campaign.id.0).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "I enter the dungeon.");
    }

    #[test]
    fn take_last_n_messages_returns_all_when_fewer_than_limit() {
        let messages: Vec<Message> = (0..5)
            .map(|i| Message::new("c".to_string(), MessageRole::User, format!("msg {i}")))
            .collect();

        let taken = take_last_n_messages(messages, 20);
        assert_eq!(taken.len(), 5);
    }

    #[test]
    fn take_last_n_messages_returns_most_recent_when_over_limit() {
        let messages: Vec<Message> = (0..25)
            .map(|i| Message::new("c".to_string(), MessageRole::User, format!("msg {i}")))
            .collect();

        let taken = take_last_n_messages(messages, 20);
        assert_eq!(taken.len(), 20);
        assert_eq!(taken[0].content, "msg 5");
        assert_eq!(taken[19].content, "msg 24");
    }
}
