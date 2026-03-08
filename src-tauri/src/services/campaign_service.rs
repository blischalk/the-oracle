use std::sync::Arc;

use crate::domain::campaign::{Campaign, CampaignState, Message};
use crate::domain::rpg_system::RpgSystem;
use crate::persistence::campaign_repository::{CampaignRepository, MessageRepository};
use crate::providers::llm_provider::ChatMessage;
use crate::services::rpg_system_registry::RpgSystemRegistry;

const MAX_CONTEXT_MESSAGES: usize = 20;

pub enum GreetingKind {
    NewCampaign,
    ResumeCampaign,
}

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

    pub fn get_campaign_state(&self, campaign_id: &str) -> anyhow::Result<CampaignState> {
        self.campaign_repository
            .find_state(campaign_id)
            .map(|opt| opt.unwrap_or_else(|| CampaignState::empty_for_campaign(campaign_id.to_string())))
    }

    pub fn save_campaign_state(&self, state: &CampaignState) -> anyhow::Result<()> {
        self.campaign_repository.save_state(state)
    }

    pub fn build_llm_context(
        &self,
        campaign_id: &str,
        rpg_system: &RpgSystem,
    ) -> anyhow::Result<Vec<ChatMessage>> {
        let all_messages = self.message_repository.find_by_campaign(campaign_id)?;
        let recent_messages = take_last_n_messages(all_messages, MAX_CONTEXT_MESSAGES);

        let mut context = vec![system_message_for(rpg_system)];
        context.extend(
            recent_messages
                .into_iter()
                .map(domain_message_to_chat_message),
        );

        Ok(context)
    }

    /// Builds context for a GM greeting: new campaign (character creation) or resume (recap + next decision).
    pub fn build_greeting_context(
        &self,
        campaign_id: &str,
        rpg_system: &RpgSystem,
        kind: GreetingKind,
    ) -> anyhow::Result<Vec<ChatMessage>> {
        let mut context = vec![system_message_for(rpg_system)];

        let instruction = match kind {
            GreetingKind::NewCampaign => {
                "A new campaign is beginning. Start character creation RIGHT NOW in this message — do not ask \
                 the player what they would like to do or what kind of adventure they want. \
                 \
                 Follow this exact sequence: \
                 1. Ask the player what they would like to name their character. This is ALWAYS the first question. \
                 2. Walk them through any remaining creation steps for this system (background, stats, equipment). \
                    Roll or assign values yourself where the rules allow — do not make the player do math. \
                    Present results narratively (e.g. 'Your hands are strong — STR 14'). \
                 3. Once the character has a name and their core attributes, immediately set the opening scene. \
                    Describe WHERE the character is, WHAT they can see, hear, and feel right now, and \
                    WHAT is immediately happening around them. Make it specific and vivid. \
                    End with a concrete situation that demands a response — not an open question like \
                    'what do you do?' in isolation, but a scene so alive the player KNOWS what they're reacting to. \
                 \
                 You are the author of this world. The player reacts to what you create. \
                 Deliver the first step of this sequence now."
            }
            GreetingKind::ResumeCampaign => {
                "The player has returned to this campaign. Give a SHORT, vivid recap (2–4 sentences) of \
                 where the story left off — what happened, where the character is, what is at stake. \
                 Then drop them straight back into the action: describe the scene as it stands RIGHT NOW \
                 and present the immediate situation in front of them. \
                 Do NOT ask 'what would you like to do?' in isolation. Present the world; let the player react. \
                 Deliver this in one message now."
            }
        };

        match kind {
            GreetingKind::NewCampaign => {
                context.push(ChatMessage {
                    role: "user".to_string(),
                    content: instruction.to_string(),
                });
            }
            GreetingKind::ResumeCampaign => {
                let all_messages = self.message_repository.find_by_campaign(campaign_id)?;
                let recent = take_last_n_messages(all_messages, MAX_CONTEXT_MESSAGES);
                context.extend(recent.into_iter().map(domain_message_to_chat_message));
                context.push(ChatMessage {
                    role: "user".to_string(),
                    content: instruction.to_string(),
                });
            }
        }

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
