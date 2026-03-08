use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CampaignId(pub String);

impl CampaignId {
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl Default for CampaignId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CampaignId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Campaign {
    pub id: CampaignId,
    pub name: String,
    pub rpg_system_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_archived: bool,
}

impl Campaign {
    pub fn created_now(name: String, rpg_system_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: CampaignId::new(),
            name,
            rpg_system_id,
            created_at: now,
            updated_at: now,
            is_archived: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageRole::System => write!(formatter, "system"),
            MessageRole::User => write!(formatter, "user"),
            MessageRole::Assistant => write!(formatter, "assistant"),
        }
    }
}

impl std::str::FromStr for MessageRole {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "system" => Ok(MessageRole::System),
            "user" => Ok(MessageRole::User),
            "assistant" => Ok(MessageRole::Assistant),
            other => Err(format!("Unknown message role: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub campaign_id: String,
    pub role: MessageRole,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub token_count: Option<u32>,
}

impl Message {
    pub fn new(campaign_id: String, role: MessageRole, content: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            campaign_id,
            role,
            content,
            created_at: Utc::now(),
            token_count: None,
        }
    }

    pub fn with_token_count(mut self, token_count: u32) -> Self {
        self.token_count = Some(token_count);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignState {
    pub campaign_id: String,
    pub character_data: serde_json::Value,
    pub notes: String,
    pub updated_at: DateTime<Utc>,
}

impl CampaignState {
    pub fn empty_for_campaign(campaign_id: String) -> Self {
        Self {
            campaign_id,
            character_data: serde_json::Value::Object(serde_json::Map::new()),
            notes: String::new(),
            updated_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn campaign_created_now_sets_timestamps_and_defaults() {
        let campaign = Campaign::created_now("Test".to_string(), "dnd5e".to_string());

        assert_eq!(campaign.name, "Test");
        assert_eq!(campaign.rpg_system_id, "dnd5e");
        assert!(!campaign.is_archived);
        assert!(!campaign.id.0.is_empty());
        assert_eq!(campaign.created_at, campaign.updated_at);
    }

    #[test]
    fn message_role_round_trips_through_string() {
        use std::str::FromStr;

        assert_eq!(MessageRole::from_str("system").unwrap(), MessageRole::System);
        assert_eq!(MessageRole::from_str("user").unwrap(), MessageRole::User);
        assert_eq!(
            MessageRole::from_str("assistant").unwrap(),
            MessageRole::Assistant
        );
        assert!(MessageRole::from_str("unknown").is_err());
    }

    #[test]
    fn message_new_generates_unique_ids() {
        let first = Message::new("camp1".to_string(), MessageRole::User, "Hello".to_string());
        let second = Message::new("camp1".to_string(), MessageRole::User, "Hello".to_string());

        assert_ne!(first.id, second.id);
    }

    #[test]
    fn message_with_token_count_sets_field() {
        let message =
            Message::new("camp1".to_string(), MessageRole::Assistant, "Reply".to_string())
                .with_token_count(42);

        assert_eq!(message.token_count, Some(42));
    }
}
