use std::sync::{Arc, Mutex};

use anyhow::Context;
use rusqlite::Connection;

#[derive(Clone)]
pub struct Database {
    pub connection: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn open(path: &str) -> anyhow::Result<Self> {
        let connection = Connection::open(path)
            .with_context(|| format!("Failed to open database at {path}"))?;

        run_migrations(&connection)?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn open_in_memory() -> anyhow::Result<Self> {
        let connection =
            Connection::open_in_memory().context("Failed to open in-memory database")?;

        run_migrations(&connection)?;

        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }
}

fn run_migrations(connection: &Connection) -> anyhow::Result<()> {
    connection
        .execute_batch(MIGRATION_SQL)
        .context("Failed to run database migrations")?;
    Ok(())
}

const MIGRATION_SQL: &str = "
CREATE TABLE IF NOT EXISTS campaigns (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    rpg_system_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    is_archived INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    campaign_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    token_count INTEGER,
    FOREIGN KEY (campaign_id) REFERENCES campaigns(id)
);

CREATE TABLE IF NOT EXISTS campaign_state (
    campaign_id TEXT PRIMARY KEY,
    character_data TEXT NOT NULL DEFAULT '{}',
    notes TEXT NOT NULL DEFAULT '',
    updated_at TEXT NOT NULL,
    FOREIGN KEY (campaign_id) REFERENCES campaigns(id)
);

CREATE TABLE IF NOT EXISTS app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_in_memory_succeeds() {
        let database = Database::open_in_memory();
        assert!(database.is_ok());
    }

    #[test]
    fn migrations_create_expected_tables() {
        let database = Database::open_in_memory().unwrap();
        let connection = database.connection.lock().unwrap();

        let table_names: Vec<String> = {
            let mut statement = connection
                .prepare(
                    "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name",
                )
                .unwrap();

            statement
                .query_map([], |row| row.get(0))
                .unwrap()
                .map(|result| result.unwrap())
                .collect()
        };

        assert!(table_names.contains(&"campaigns".to_string()));
        assert!(table_names.contains(&"messages".to_string()));
        assert!(table_names.contains(&"campaign_state".to_string()));
        assert!(table_names.contains(&"app_settings".to_string()));
    }

    #[test]
    fn migrations_are_idempotent() {
        let database = Database::open_in_memory().unwrap();
        let connection = database.connection.lock().unwrap();

        // Running migrations a second time must not fail.
        let result = run_migrations(&connection);
        assert!(result.is_ok());
    }
}
