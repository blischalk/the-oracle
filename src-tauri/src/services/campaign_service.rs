use std::sync::Arc;

use crate::domain::campaign::{Campaign, Message};
use crate::domain::rpg_system::RpgSystem;
use crate::persistence::campaign_repository::{CampaignRepository, MessageRepository};
use crate::providers::llm_provider::ChatMessage;
use crate::services::rpg_system_registry::RpgSystemRegistry;

const MAX_CONTEXT_MESSAGES: usize = 20;

pub struct CampaignService {
    campaign_repository: Arc<CampaignRepository>,
    message_repository: Arc<MessageRepository>,
    rpg_registry: Arc<RpgSystemRegistry>,
}

impl CampaignService {
    pub fn new(
        campaign_repository: Arc<CampaignRepository>,
        message_repository: Arc<MessageRepository>,
        rpg_registry: Arc<RpgSystemRegistry>,
    ) -> Self {
        Self {
            campaign_repository,
            message_repository,
            rpg_registry,
        }
    }

    pub fn create_campaign(
        &self,
        name: &str,
        rpg_system_id: &str,
    ) -> anyhow::Result<Campaign> {
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

    pub fn save_message(&self, message: &Message) -> anyhow::Result<()> {
        self.message_repository.save(message)
    }

    pub fn get_messages(&self, campaign_id: &str) -> anyhow::Result<Vec<Message>> {
        self.message_repository.find_by_campaign(campaign_id)
    }

    pub fn build_llm_context(
        &self,
        campaign_id: &str,
        rpg_system: &RpgSystem,
    ) -> anyhow::Result<Vec<ChatMessage>> {
        let all_messages = self.message_repository.find_by_campaign(campaign_id)?;
        let recent_messages = take_last_n_messages(all_messages, MAX_CONTEXT_MESSAGES);

        let mut context = vec![system_message_for(rpg_system)];
        context.extend(recent_messages.into_iter().map(domain_message_to_chat_message));

        Ok(context)
    }

    pub fn rpg_registry(&self) -> &RpgSystemRegistry {
        &self.rpg_registry
    }
}

fn system_message_for(rpg_system: &RpgSystem) -> ChatMessage {
    ChatMessage {
        role: "system".to_string(),
        content: rpg_system.system_prompt.clone(),
    }
}

fn take_last_n_messages(messages: Vec<Message>, count: usize) -> Vec<Message> {
    if messages.len() <= count {
        messages
    } else {
        messages.into_iter().rev().take(count).collect::<Vec<_>>().into_iter().rev().collect()
    }
}

fn domain_message_to_chat_message(message: Message) -> ChatMessage {
    ChatMessage {
        role: message.role.to_string(),
        content: message.content,
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
        let registry = Arc::new(RpgSystemRegistry::load(std::path::Path::new("/nonexistent")).unwrap());

        CampaignService::new(campaign_repo, message_repo, registry)
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
