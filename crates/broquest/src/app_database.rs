use gpui::{App, Global};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use sqlx::{ConnectOptions, Row};
use std::path::PathBuf;
use std::str::FromStr;

/// Application database for persistance
#[derive(Clone)]
pub struct AppDatabase {
    pool: SqlitePool,
}

#[derive(Debug, Clone)]
pub struct CollectionData {
    #[allow(dead_code)]
    pub id: Option<i64>,
    pub name: String,
    pub path: String,
    pub position: i32,
    #[allow(dead_code)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[allow(dead_code)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct TabData {
    pub id: Option<i64>,
    pub title: String,
    pub collection_id: Option<i64>,
    pub file_path: Option<String>,
    pub request_data: String, // JSON serialized RequestData
    pub environment: Option<String>,
    pub position: i32,
    #[allow(dead_code)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[allow(dead_code)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct UserSetting {
    pub theme: String,
}

impl Global for AppDatabase {}

impl AppDatabase {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let db_path = Self::app_db_path();

        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))?
            .create_if_missing(true)
            .disable_statement_logging();

        let pool = SqlitePool::connect_with(options).await?;

        let mut db = Self { pool };
        db.init_schema().await?;
        Ok(db)
    }

    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    fn app_db_path() -> PathBuf {
        let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("broquest");
        std::fs::create_dir_all(&path).ok();
        path.push("broquest.db");
        path
    }

    async fn init_schema(&mut self) -> Result<(), sqlx::Error> {
        // Collections table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS collections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                position INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Tabs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tabs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                collection_id INTEGER,
                file_path TEXT,
                request_data TEXT NOT NULL,
                environment TEXT,
                position INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Settings table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_settings (
                id INTEGER PRIMARY KEY,
                theme TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Add new columns if they don't exist (for migration)
        // Check if collection_id column exists
        let result = sqlx::query("PRAGMA table_info(tabs)")
            .fetch_all(&self.pool)
            .await?;

        let has_collection_id = result
            .iter()
            .any(|row| row.get::<Option<String>, _>("name") == Some("collection_id".to_string()));

        if !has_collection_id {
            // Migrate existing tabs table
            sqlx::query(
                r#"
                ALTER TABLE tabs ADD COLUMN collection_id INTEGER;
                ALTER TABLE tabs ADD COLUMN file_path TEXT;
                ALTER TABLE tabs ADD COLUMN request_data TEXT NOT NULL DEFAULT '{}';
                ALTER TABLE tabs ADD COLUMN environment TEXT;
                "#,
            )
            .execute(&self.pool)
            .await?;

            // Update existing tabs to have empty JSON request_data
            sqlx::query(
                "UPDATE tabs SET request_data = '{}' WHERE request_data IS NULL OR request_data = ''"
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    // Collections operations
    pub async fn save_collection(&self, collection: &CollectionData) -> Result<i64, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();

        // Upsert collection - insert new or update existing based on unique path constraint
        let result = sqlx::query(
            r#"
            INSERT INTO collections (name, path, position, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(path) DO UPDATE SET
                name = EXCLUDED.name,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(&collection.name)
        .bind(&collection.path)
        .bind(collection.position)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn load_collections(&self) -> Result<Vec<CollectionData>, sqlx::Error> {
        // TODO: allow re-ordering via position
        let rows = sqlx::query(
            r#"
            SELECT c.id, c.name, c.path, c.position, c.created_at, c.updated_at
            FROM collections c
            ORDER BY c.id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut collections = Vec::new();
        for row in rows {
            collections.push(CollectionData {
                id: Some(row.get("id")),
                name: row.get("name"),
                path: row.get("path"),
                position: row.get("position"),
                created_at: chrono::DateTime::from_timestamp(row.get("created_at"), 0)
                    .unwrap_or_default(),
                updated_at: chrono::DateTime::from_timestamp(row.get("updated_at"), 0)
                    .unwrap_or_default(),
            });
        }

        Ok(collections)
    }

    /// Delete a collection from the database by path
    pub async fn delete_collection(&self, path: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM collections WHERE path = ?")
            .bind(path)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn save_user_settings(&self, user_setting: &UserSetting) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().timestamp();

        // Update existing tab
        sqlx::query(
            r#"
                INSERT INTO user_settings (id, theme, updated_at) VALUES (1, ?, ?)
                    ON CONFLICT (id)
                    DO UPDATE
                    SET theme = excluded.theme,
                    updated_at = excluded.updated_at
                "#,
        )
        .bind(&user_setting.theme)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_user_settings(&self) -> Result<Option<UserSetting>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT ut.theme
            FROM user_settings ut
            WHERE ut.id = 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(UserSetting { theme: row.get(0) }))
        } else {
            Ok(None)
        }
    }

    #[allow(dead_code)]
    pub async fn save_tab(&self, tab: &TabData) -> Result<i64, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();

        if let Some(id) = tab.id {
            // Update existing tab
            sqlx::query(
                r#"
                UPDATE tabs
                SET title = ?, collection_id = ?, file_path = ?, request_data = ?, environment = ?, position = ?, updated_at = ?
                WHERE id = ?
                "#,
            )
            .bind(&tab.title)
            .bind(tab.collection_id)
            .bind(&tab.file_path)
            .bind(&tab.request_data)
            .bind(&tab.environment)
            .bind(tab.position)
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await?;
            Ok(id)
        } else {
            // Insert new tab
            let result = sqlx::query(
                r#"
                INSERT INTO tabs (title, collection_id, file_path, request_data, environment, position, created_at, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&tab.title)
            .bind(tab.collection_id)
            .bind(&tab.file_path)
            .bind(&tab.request_data)
            .bind(&tab.environment)
            .bind(tab.position)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await?;

            Ok(result.last_insert_rowid())
        }
    }

    pub async fn load_tabs(&self) -> Result<Vec<TabData>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT t.id, t.title, t.collection_id, t.file_path, t.request_data, t.environment, t.position, t.created_at, t.updated_at
            FROM tabs t
            ORDER BY t.position ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let tabs = rows
            .into_iter()
            .map(|row| TabData {
                id: Some(row.get("id")),
                title: row.get("title"),
                collection_id: row.get("collection_id"),
                file_path: row.get("file_path"),
                request_data: row.get("request_data"),
                environment: row.get("environment"),
                position: row.get("position"),
                created_at: chrono::DateTime::from_timestamp(row.get("created_at"), 0)
                    .unwrap_or_default(),
                updated_at: chrono::DateTime::from_timestamp(row.get("updated_at"), 0)
                    .unwrap_or_default(),
            })
            .collect();

        Ok(tabs)
    }

    #[allow(dead_code)]
    pub async fn delete_tab(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM tabs WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
