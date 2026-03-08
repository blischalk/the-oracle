use std::str::FromStr;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use rusqlite::{params, Connection};

use crate::domain::campaign::{Campaign, CampaignId, CampaignState, Message, MessageRole};

pub struct CampaignRepository {
    connection: Arc<Mutex<Connection>>,
}

impl CampaignRepository {
    pub fn new(connection: Arc<Mutex<Connection>>) -> Self {
        Self { connection }
    }

    pub fn create(&self, campaign: &Campaign) -> anyhow::Result<()> {
        let connection = self.connection.lock().unwrap();
        connection
            .execute(
                "INSERT INTO campaigns (id, name, rpg_system_id, created_at, updated_at, is_archived)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    campaign.id.0,
                    campaign.name,
                    campaign.rpg_system_id,
                    campaign.created_at.to_rfc3339(),
                    campaign.updated_at.to_rfc3339(),
                    campaign.is_archived as i32,
                ],
            )
            .context("Failed to insert campaign")?;
        Ok(())
    }

    pub fn find_by_id(&self, id: &str) -> anyhow::Result<Option<Campaign>> {
        let connection = self.connection.lock().unwrap();
        let mut statement = connection
            .prepare(
                "SELECT id, name, rpg_system_id, created_at, updated_at, is_archived
                 FROM campaigns WHERE id = ?1",
            )
            .context("Failed to prepare find_by_id statement")?;

        let mut rows = statement
            .query_map(params![id], row_to_campaign)
            .context("Failed to query campaign by id")?;

        match rows.next() {
            Some(result) => Ok(Some(result.context("Failed to map campaign row")?)),
            None => Ok(None),
        }
    }

    pub fn list_active(&self) -> anyhow::Result<Vec<Campaign>> {
        let connection = self.connection.lock().unwrap();
        let mut statement = connection
            .prepare(
                "SELECT id, name, rpg_system_id, created_at, updated_at, is_archived
                 FROM campaigns WHERE is_archived = 0 ORDER BY created_at DESC",
            )
            .context("Failed to prepare list_active statement")?;

        let campaigns = statement
            .query_map([], row_to_campaign)
            .context("Failed to query active campaigns")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to map campaign rows")?;

        Ok(campaigns)
    }

    pub fn archive(&self, id: &str) -> anyhow::Result<()> {
        let connection = self.connection.lock().unwrap();
        connection
            .execute(
                "UPDATE campaigns SET is_archived = 1 WHERE id = ?1",
                params![id],
            )
            .context("Failed to archive campaign")?;
        Ok(())
    }

    /// Permanently deletes a campaign and all its messages and state.
    pub fn delete(&self, id: &str) -> anyhow::Result<()> {
        let mut connection = self.connection.lock().unwrap();
        let tx = connection
            .transaction()
            .context("Failed to start transaction")?;
        tx.execute("DELETE FROM messages WHERE campaign_id = ?1", params![id])
            .context("Failed to delete campaign messages")?;
        if let Err(e) = tx.execute(
            "DELETE FROM campaign_state WHERE campaign_id = ?1",
            params![id],
        ) {
            let msg = e.to_string();
            if !msg.contains("no such table") {
                return Err(e.into());
            }
        }
        tx.execute("DELETE FROM campaigns WHERE id = ?1", params![id])
            .context("Failed to delete campaign")?;
        tx.commit().context("Failed to commit delete transaction")?;
        Ok(())
    }

    pub fn update_name(&self, id: &str, name: &str) -> anyhow::Result<()> {
        let connection = self.connection.lock().unwrap();
        let updated_at = chrono::Utc::now().to_rfc3339();
        connection
            .execute(
                "UPDATE campaigns SET name = ?1, updated_at = ?2 WHERE id = ?3",
                params![name, updated_at, id],
            )
            .context("Failed to update campaign name")?;
        Ok(())
    }

    pub fn find_state(&self, campaign_id: &str) -> anyhow::Result<Option<CampaignState>> {
        let connection = self.connection.lock().unwrap();
        let mut statement = connection
            .prepare(
                "SELECT campaign_id, character_data, notes, updated_at
                 FROM campaign_state WHERE campaign_id = ?1",
            )
            .context("Failed to prepare find_state statement")?;

        let mut rows = statement
            .query_map(params![campaign_id], row_to_campaign_state)
            .context("Failed to query campaign state")?;

        match rows.next() {
            Some(result) => Ok(Some(result.context("Failed to map campaign state row")?)),
            None => Ok(None),
        }
    }

    pub fn save_state(&self, state: &CampaignState) -> anyhow::Result<()> {
        let connection = self.connection.lock().unwrap();
        let character_data_json =
            serde_json::to_string(&state.character_data).context("Failed to serialize character_data")?;
        connection
            .execute(
                "INSERT INTO campaign_state (campaign_id, character_data, notes, updated_at)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(campaign_id) DO UPDATE SET
                   character_data = excluded.character_data,
                   notes = excluded.notes,
                   updated_at = excluded.updated_at",
                params![
                    state.campaign_id,
                    character_data_json,
                    state.notes,
                    state.updated_at.to_rfc3339(),
                ],
            )
            .context("Failed to save campaign state")?;
        Ok(())
    }
}

fn row_to_campaign_state(row: &rusqlite::Row<'_>) -> rusqlite::Result<CampaignState> {
    let campaign_id: String = row.get(0)?;
    let character_data_str: String = row.get(1)?;
    let notes: String = row.get(2)?;
    let updated_at_str: String = row.get(3)?;

    let character_data: serde_json::Value = serde_json::from_str(&character_data_str)
        .unwrap_or_else(|_| serde_json::Value::Object(serde_json::Map::new()));

    let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|_| {
            rusqlite::Error::InvalidColumnType(
                3,
                "updated_at".to_string(),
                rusqlite::types::Type::Text,
            )
        })?;

    Ok(CampaignState {
        campaign_id,
        character_data,
        notes,
        updated_at,
    })
}

fn row_to_campaign(row: &rusqlite::Row<'_>) -> rusqlite::Result<Campaign> {
    let id: String = row.get(0)?;
    let name: String = row.get(1)?;
    let rpg_system_id: String = row.get(2)?;
    let created_at_str: String = row.get(3)?;
    let updated_at_str: String = row.get(4)?;
    let is_archived_int: i32 = row.get(5)?;

    let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|_| {
            rusqlite::Error::InvalidColumnType(
                3,
                "created_at".to_string(),
                rusqlite::types::Type::Text,
            )
        })?;

    let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_at_str)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|_| {
            rusqlite::Error::InvalidColumnType(
                4,
                "updated_at".to_string(),
                rusqlite::types::Type::Text,
            )
        })?;

    Ok(Campaign {
        id: CampaignId(id),
        name,
        rpg_system_id,
        created_at,
        updated_at,
        is_archived: is_archived_int != 0,
    })
}

pub struct MessageRepository {
    connection: Arc<Mutex<Connection>>,
}

impl MessageRepository {
    pub fn new(connection: Arc<Mutex<Connection>>) -> Self {
        Self { connection }
    }

    pub fn save(&self, message: &Message) -> anyhow::Result<()> {
        let connection = self.connection.lock().unwrap();
        connection
            .execute(
                "INSERT INTO messages (id, campaign_id, role, content, created_at, token_count)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    message.id,
                    message.campaign_id,
                    message.role.to_string(),
                    message.content,
                    message.created_at.to_rfc3339(),
                    message.token_count.map(|count| count as i64),
                ],
            )
            .context("Failed to insert message")?;
        Ok(())
    }

    pub fn find_by_campaign(&self, campaign_id: &str) -> anyhow::Result<Vec<Message>> {
        let connection = self.connection.lock().unwrap();
        let mut statement = connection
            .prepare(
                "SELECT id, campaign_id, role, content, created_at, token_count
                 FROM messages WHERE campaign_id = ?1 ORDER BY created_at ASC",
            )
            .context("Failed to prepare find_by_campaign statement")?;

        let messages = statement
            .query_map(params![campaign_id], row_to_message)
            .context("Failed to query messages by campaign")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to map message rows")?;

        Ok(messages)
    }

    pub fn count_for_campaign(&self, campaign_id: &str) -> anyhow::Result<u32> {
        let connection = self.connection.lock().unwrap();
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM messages WHERE campaign_id = ?1",
                params![campaign_id],
                |row| row.get(0),
            )
            .context("Failed to count messages for campaign")?;

        Ok(count as u32)
    }
}

fn row_to_message(row: &rusqlite::Row<'_>) -> rusqlite::Result<Message> {
    let id: String = row.get(0)?;
    let campaign_id: String = row.get(1)?;
    let role_str: String = row.get(2)?;
    let content: String = row.get(3)?;
    let created_at_str: String = row.get(4)?;
    let token_count_raw: Option<i64> = row.get(5)?;

    let role = MessageRole::from_str(&role_str).map_err(|_| {
        rusqlite::Error::InvalidColumnType(2, "role".to_string(), rusqlite::types::Type::Text)
    })?;

    let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|_| {
            rusqlite::Error::InvalidColumnType(
                4,
                "created_at".to_string(),
                rusqlite::types::Type::Text,
            )
        })?;

    Ok(Message {
        id,
        campaign_id,
        role,
        content,
        created_at,
        token_count: token_count_raw.map(|count| count as u32),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::campaign::{Campaign, Message, MessageRole};
    use crate::persistence::database::Database;

    fn build_repositories() -> (CampaignRepository, MessageRepository) {
        let database = Database::open_in_memory().unwrap();
        let campaign_repo = CampaignRepository::new(database.connection.clone());
        let message_repo = MessageRepository::new(database.connection.clone());
        (campaign_repo, message_repo)
    }

    fn make_campaign(name: &str) -> Campaign {
        Campaign::created_now(name.to_string(), "dnd5e".to_string())
    }

    #[test]
    fn create_and_find_by_id_round_trips_campaign() {
        let (campaign_repo, _) = build_repositories();
        let campaign = make_campaign("Adventure One");
        campaign_repo.create(&campaign).unwrap();

        let found = campaign_repo
            .find_by_id(&campaign.id.0)
            .unwrap()
            .expect("campaign should be found");

        assert_eq!(found.name, "Adventure One");
        assert_eq!(found.rpg_system_id, "dnd5e");
        assert!(!found.is_archived);
    }

    #[test]
    fn find_by_id_returns_none_for_missing_campaign() {
        let (campaign_repo, _) = build_repositories();
        let result = campaign_repo.find_by_id("nonexistent-id").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn list_active_excludes_archived_campaigns() {
        let (campaign_repo, _) = build_repositories();
        let active = make_campaign("Active");
        let archived = make_campaign("Archived");

        campaign_repo.create(&active).unwrap();
        campaign_repo.create(&archived).unwrap();
        campaign_repo.archive(&archived.id.0).unwrap();

        let active_list = campaign_repo.list_active().unwrap();
        assert_eq!(active_list.len(), 1);
        assert_eq!(active_list[0].name, "Active");
    }

    #[test]
    fn save_and_find_messages_round_trips() {
        let (campaign_repo, message_repo) = build_repositories();
        let campaign = make_campaign("Campaign");
        campaign_repo.create(&campaign).unwrap();

        let message = Message::new(
            campaign.id.0.clone(),
            MessageRole::User,
            "Hello, Oracle.".to_string(),
        );
        message_repo.save(&message).unwrap();

        let found = message_repo.find_by_campaign(&campaign.id.0).unwrap();

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].content, "Hello, Oracle.");
        assert_eq!(found[0].role, MessageRole::User);
    }

    #[test]
    fn count_for_campaign_returns_correct_count() {
        let (campaign_repo, message_repo) = build_repositories();
        let campaign = make_campaign("Campaign");
        campaign_repo.create(&campaign).unwrap();

        message_repo
            .save(&Message::new(
                campaign.id.0.clone(),
                MessageRole::User,
                "First".to_string(),
            ))
            .unwrap();
        message_repo
            .save(&Message::new(
                campaign.id.0.clone(),
                MessageRole::Assistant,
                "Second".to_string(),
            ))
            .unwrap();

        let count = message_repo.count_for_campaign(&campaign.id.0).unwrap();
        assert_eq!(count, 2);
    }
}
