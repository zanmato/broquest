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
    /// On-disk format, e.g. "broquest" or "opencollection".
    /// See [`crate::collections::CollectionFormat`].
    pub format: String,
    #[allow(dead_code)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[allow(dead_code)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct UserSetting {
    pub theme: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HistoryEntry {
    pub id: Option<i64>,
    pub method: String,
    pub url: String,
    pub status_code: Option<i32>,
    pub latency_ms: Option<i64>,
    pub response_size: Option<i64>,
    pub request_name: Option<String>,
    pub collection_path: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
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

        // Migration: add the per-collection format column for databases created
        // before OpenCollection support. Errors (e.g. column already exists) are
        // ignored so this is safe to run on every startup.
        let _ = sqlx::query(
            "ALTER TABLE collections ADD COLUMN format TEXT NOT NULL DEFAULT 'broquest'",
        )
        .execute(&self.pool)
        .await;

        // User settings table (legacy)
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

        // Key-value settings table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Request history table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS request_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at INTEGER NOT NULL,
                method TEXT NOT NULL,
                url TEXT NOT NULL,
                status_code INTEGER,
                latency_ms INTEGER,
                response_size INTEGER,
                request_name TEXT,
                collection_path TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_history_created_at ON request_history(created_at DESC)",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Collections operations
    pub async fn save_collection(&self, collection: &CollectionData) -> Result<i64, sqlx::Error> {
        let now = chrono::Utc::now().timestamp();

        // Upsert collection - insert new or update existing based on unique path constraint
        let result = sqlx::query(
            r#"
            INSERT INTO collections (name, path, position, format, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(path) DO UPDATE SET
                name = EXCLUDED.name,
                format = EXCLUDED.format,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(&collection.name)
        .bind(&collection.path)
        .bind(collection.position)
        .bind(&collection.format)
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
            SELECT c.id, c.name, c.path, c.position, c.format, c.created_at, c.updated_at
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
                format: row.get("format"),
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

    // Key-value settings operations
    pub async fn save_setting(&self, key: &str, value: &str) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO settings (key, value, created_at, updated_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(key)
        .bind(value)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn load_all_settings(&self) -> Result<Vec<(String, String)>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT key, value FROM settings
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| (r.get(0), r.get(1))).collect())
    }

    // History operations
    pub async fn insert_history(&self, entry: &HistoryEntry) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO request_history (created_at, method, url, status_code, latency_ms, response_size, request_name, collection_path)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(now)
        .bind(&entry.method)
        .bind(&entry.url)
        .bind(entry.status_code)
        .bind(entry.latency_ms)
        .bind(entry.response_size)
        .bind(&entry.request_name)
        .bind(&entry.collection_path)
        .execute(&self.pool)
        .await?;

        // Prune old entries, keeping the most recent 500
        sqlx::query(
            r#"
            DELETE FROM request_history WHERE id NOT IN (
                SELECT id FROM request_history ORDER BY created_at DESC LIMIT 500
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn load_recent_history(&self, limit: i64) -> Result<Vec<HistoryEntry>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, created_at, method, url, status_code, latency_ms, response_size, request_name, collection_path
            FROM request_history
            ORDER BY created_at DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(HistoryEntry {
                id: Some(row.get("id")),
                method: row.get("method"),
                url: row.get("url"),
                status_code: row.get("status_code"),
                latency_ms: row.get("latency_ms"),
                response_size: row.get("response_size"),
                request_name: row.get("request_name"),
                collection_path: row.get("collection_path"),
                created_at: chrono::DateTime::from_timestamp(row.get("created_at"), 0)
                    .unwrap_or_default(),
            });
        }

        Ok(entries)
    }

    pub async fn clear_history(&self) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM request_history")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
