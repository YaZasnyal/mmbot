use crate::error::ThreadBotError;
use crate::store::ThreadStore;
use crate::types::{
    ChannelCheckpoint, ThreadLink, ThreadMessageRecord, ThreadRecord, UpsertThread,
    UpsertThreadLink, UpsertThreadMessage,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

/// Production implementation of ThreadStore backed by PostgreSQL
pub struct PgThreadStore {
    pool: PgPool,
}

impl PgThreadStore {
    /// Create a new PgThreadStore and run migrations
    pub async fn new(pool: PgPool) -> Result<Self, ThreadBotError> {
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| ThreadBotError::Internal(Box::new(e)))?;
        Ok(Self { pool })
    }
}

// Row types for sqlx queries
#[derive(FromRow)]
struct ThreadRow {
    thread_id: String,
    root_post_id: String,
    channel_id: String,
    creator_user_id: String,
    thread_kind: Option<String>,
    metadata: serde_json::Value,
    last_seen_post_id: Option<String>,
    last_seen_post_at: Option<DateTime<Utc>>,
    last_processed_post_id: Option<String>,
    last_processed_post_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<ThreadRow> for ThreadRecord {
    fn from(row: ThreadRow) -> Self {
        Self {
            thread_id: row.thread_id,
            root_post_id: row.root_post_id,
            channel_id: row.channel_id,
            creator_user_id: row.creator_user_id,
            thread_kind: row.thread_kind,
            metadata: row.metadata,
            last_seen_post_id: row.last_seen_post_id,
            last_seen_post_at: row.last_seen_post_at,
            last_processed_post_id: row.last_processed_post_id,
            last_processed_post_at: row.last_processed_post_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(FromRow)]
struct ThreadLinkRow {
    source_thread_id: String,
    link_kind: String,
    target_thread_id: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<ThreadLinkRow> for ThreadLink {
    fn from(row: ThreadLinkRow) -> Self {
        Self {
            source_thread_id: row.source_thread_id,
            link_kind: row.link_kind,
            target_thread_id: row.target_thread_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(FromRow)]
struct MessageRow {
    post_id: String,
    thread_id: String,
    user_id: String,
    is_bot_message: bool,
    root_id: Option<String>,
    parent_post_id: Option<String>,
    metadata: serde_json::Value,
    post_created_at: DateTime<Utc>,
    post_updated_at: Option<DateTime<Utc>>,
    post_deleted_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<MessageRow> for ThreadMessageRecord {
    fn from(row: MessageRow) -> Self {
        Self {
            post_id: row.post_id,
            thread_id: row.thread_id,
            user_id: row.user_id,
            is_bot_message: row.is_bot_message,
            root_id: row.root_id,
            parent_post_id: row.parent_post_id,
            metadata: row.metadata,
            post_created_at: row.post_created_at,
            post_updated_at: row.post_updated_at,
            post_deleted_at: row.post_deleted_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

const THREAD_COLUMNS: &str = r#"
    thread_id, root_post_id, channel_id, creator_user_id, thread_kind, metadata,
    last_seen_post_id, last_seen_post_at,
    last_processed_post_id, last_processed_post_at,
    created_at, updated_at
"#;

const MESSAGE_COLUMNS: &str = r#"
    post_id, thread_id, user_id, is_bot_message, root_id, parent_post_id,
    metadata, post_created_at, post_updated_at, post_deleted_at,
    created_at, updated_at
"#;

const THREAD_LINK_COLUMNS: &str = r#"
    source_thread_id, link_kind, target_thread_id, created_at, updated_at
"#;

#[async_trait]
impl ThreadStore for PgThreadStore {
    async fn upsert_thread(&self, input: UpsertThread) -> Result<ThreadRecord, ThreadBotError> {
        let now = Utc::now();

        let sql = format!(
            r#"
            INSERT INTO threads (
                thread_id, root_post_id, channel_id, creator_user_id,
                thread_kind, metadata, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
            ON CONFLICT (thread_id) DO UPDATE SET
                thread_kind = EXCLUDED.thread_kind,
                metadata = EXCLUDED.metadata,
                updated_at = EXCLUDED.updated_at
            RETURNING {THREAD_COLUMNS}
            "#
        );

        let row: ThreadRow = sqlx::query_as(&sql)
            .bind(&input.thread_id)
            .bind(&input.root_post_id)
            .bind(&input.channel_id)
            .bind(&input.creator_user_id)
            .bind(&input.thread_kind)
            .bind(&input.metadata)
            .bind(now)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.into())
    }

    async fn get_thread(&self, thread_id: &str) -> Result<Option<ThreadRecord>, ThreadBotError> {
        let sql = format!("SELECT {THREAD_COLUMNS} FROM threads WHERE thread_id = $1");

        let row: Option<ThreadRow> = sqlx::query_as(&sql)
            .bind(thread_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(Into::into))
    }

    async fn get_thread_by_post(
        &self,
        post_id: &str,
    ) -> Result<Option<ThreadRecord>, ThreadBotError> {
        let sql = format!(
            r#"
            SELECT {THREAD_COLUMNS}
            FROM threads
            WHERE root_post_id = $1
               OR thread_id IN (
                   SELECT thread_id FROM thread_messages WHERE post_id = $1
               )
            "#
        );

        let row: Option<ThreadRow> = sqlx::query_as(&sql)
            .bind(post_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(Into::into))
    }

    async fn list_threads(
        &self,
        updated_after: Option<DateTime<Utc>>,
        updated_before: Option<DateTime<Utc>>,
    ) -> Result<Vec<ThreadRecord>, ThreadBotError> {
        let sql = format!(
            "SELECT {THREAD_COLUMNS} FROM threads \
             WHERE ($1::timestamptz IS NULL OR updated_at > $1) \
             AND ($2::timestamptz IS NULL OR updated_at < $2) \
             ORDER BY updated_at DESC"
        );

        let rows: Vec<ThreadRow> = sqlx::query_as(&sql)
            .bind(updated_after)
            .bind(updated_before)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn set_thread_metadata(
        &self,
        thread_id: &str,
        metadata: serde_json::Value,
    ) -> Result<(), ThreadBotError> {
        let now = Utc::now();

        let result =
            sqlx::query("UPDATE threads SET metadata = $2, updated_at = $3 WHERE thread_id = $1")
                .bind(thread_id)
                .bind(metadata)
                .bind(now)
                .execute(&self.pool)
                .await?;

        if result.rows_affected() == 0 {
            return Err(ThreadBotError::ThreadNotFound(thread_id.to_string()));
        }
        Ok(())
    }

    async fn upsert_thread_link(
        &self,
        input: UpsertThreadLink,
    ) -> Result<ThreadLink, ThreadBotError> {
        let now = Utc::now();
        let sql = format!(
            r#"
            INSERT INTO thread_links (
                source_thread_id, link_kind, target_thread_id, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $4)
            ON CONFLICT (source_thread_id, target_thread_id) DO UPDATE SET
                link_kind = EXCLUDED.link_kind,
                updated_at = EXCLUDED.updated_at
            RETURNING {THREAD_LINK_COLUMNS}
            "#
        );

        let row: ThreadLinkRow = sqlx::query_as(&sql)
            .bind(&input.source_thread_id)
            .bind(&input.link_kind)
            .bind(&input.target_thread_id)
            .bind(now)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.into())
    }

    async fn get_thread_link(
        &self,
        source_thread_id: &str,
        target_thread_id: &str,
    ) -> Result<Option<ThreadLink>, ThreadBotError> {
        let sql = format!(
            "SELECT {THREAD_LINK_COLUMNS} FROM thread_links \
             WHERE source_thread_id = $1 AND target_thread_id = $2"
        );

        let row: Option<ThreadLinkRow> = sqlx::query_as(&sql)
            .bind(source_thread_id)
            .bind(target_thread_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(Into::into))
    }

    async fn list_thread_links(
        &self,
        source_thread_id: &str,
    ) -> Result<Vec<ThreadLink>, ThreadBotError> {
        let sql = format!(
            "SELECT {THREAD_LINK_COLUMNS} FROM thread_links \
             WHERE source_thread_id = $1 ORDER BY link_kind ASC, target_thread_id ASC"
        );

        let rows: Vec<ThreadLinkRow> = sqlx::query_as(&sql)
            .bind(source_thread_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn list_reverse_thread_links(
        &self,
        target_thread_id: &str,
    ) -> Result<Vec<ThreadLink>, ThreadBotError> {
        let sql = format!(
            "SELECT {THREAD_LINK_COLUMNS} FROM thread_links \
             WHERE target_thread_id = $1 ORDER BY source_thread_id ASC, link_kind ASC"
        );

        let rows: Vec<ThreadLinkRow> = sqlx::query_as(&sql)
            .bind(target_thread_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update_thread_seen(
        &self,
        thread_id: &str,
        post_id: &str,
        seen_at: DateTime<Utc>,
    ) -> Result<(), ThreadBotError> {
        let now = Utc::now();

        let result = sqlx::query(
            "UPDATE threads SET last_seen_post_id = $2, last_seen_post_at = $3, updated_at = $4 WHERE thread_id = $1",
        )
        .bind(thread_id)
        .bind(post_id)
        .bind(seen_at)
        .bind(now)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(ThreadBotError::ThreadNotFound(thread_id.to_string()));
        }
        Ok(())
    }

    async fn update_thread_processed(
        &self,
        thread_id: &str,
        post_id: &str,
        processed_at: DateTime<Utc>,
    ) -> Result<(), ThreadBotError> {
        let now = Utc::now();

        let result = sqlx::query(
            "UPDATE threads SET last_processed_post_id = $2, last_processed_post_at = $3, updated_at = $4 WHERE thread_id = $1",
        )
        .bind(thread_id)
        .bind(post_id)
        .bind(processed_at)
        .bind(now)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(ThreadBotError::ThreadNotFound(thread_id.to_string()));
        }
        Ok(())
    }

    async fn upsert_message(
        &self,
        input: UpsertThreadMessage,
    ) -> Result<ThreadMessageRecord, ThreadBotError> {
        let now = Utc::now();

        let sql = format!(
            r#"
            INSERT INTO thread_messages (post_id, thread_id, user_id, is_bot_message, root_id, parent_post_id,
                                         metadata, post_created_at, post_updated_at, post_deleted_at,
                                         created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $11)
            ON CONFLICT (post_id) DO UPDATE SET
                is_bot_message = EXCLUDED.is_bot_message,
                metadata = EXCLUDED.metadata,
                post_updated_at = EXCLUDED.post_updated_at,
                post_deleted_at = EXCLUDED.post_deleted_at,
                updated_at = EXCLUDED.updated_at
            RETURNING {MESSAGE_COLUMNS}
            "#
        );

        let row: MessageRow = sqlx::query_as(&sql)
            .bind(&input.post_id)
            .bind(&input.thread_id)
            .bind(&input.user_id)
            .bind(input.is_bot_message)
            .bind(&input.root_id)
            .bind(&input.parent_post_id)
            .bind(&input.metadata)
            .bind(input.post_created_at)
            .bind(input.post_updated_at)
            .bind(input.post_deleted_at)
            .bind(now)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.into())
    }

    async fn get_message(
        &self,
        post_id: &str,
    ) -> Result<Option<ThreadMessageRecord>, ThreadBotError> {
        let sql = format!("SELECT {MESSAGE_COLUMNS} FROM thread_messages WHERE post_id = $1");

        let row: Option<MessageRow> = sqlx::query_as(&sql)
            .bind(post_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(Into::into))
    }

    async fn list_thread_messages(
        &self,
        thread_id: &str,
    ) -> Result<Vec<ThreadMessageRecord>, ThreadBotError> {
        let sql = format!(
            "SELECT {MESSAGE_COLUMNS} FROM thread_messages WHERE thread_id = $1 ORDER BY post_created_at ASC"
        );

        let rows: Vec<MessageRow> = sqlx::query_as(&sql)
            .bind(thread_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn set_message_metadata(
        &self,
        post_id: &str,
        metadata: serde_json::Value,
    ) -> Result<(), ThreadBotError> {
        let now = Utc::now();

        let result = sqlx::query(
            "UPDATE thread_messages SET metadata = $2, updated_at = $3 WHERE post_id = $1",
        )
        .bind(post_id)
        .bind(metadata)
        .bind(now)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(ThreadBotError::MessageNotFound(post_id.to_string()));
        }
        Ok(())
    }

    // ── Channel checkpoints ─────────────────────────────────────────────

    async fn list_channel_checkpoints(&self) -> Result<Vec<ChannelCheckpoint>, ThreadBotError> {
        let rows: Vec<CheckpointRow> = sqlx::query_as(
            "SELECT channel_id, last_seen_post_id, last_seen_post_at, updated_at, is_reconciled FROM channel_checkpoints",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn upsert_channel_checkpoint(
        &self,
        channel_id: &str,
        last_seen_post_id: &str,
        last_seen_post_at: DateTime<Utc>,
    ) -> Result<(), ThreadBotError> {
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO channel_checkpoints (channel_id, last_seen_post_id, last_seen_post_at, updated_at, is_reconciled)
            VALUES ($1, $2, $3, $4, true)
            ON CONFLICT (channel_id) DO UPDATE SET
                last_seen_post_id = CASE
                    WHEN EXCLUDED.last_seen_post_at >= channel_checkpoints.last_seen_post_at
                    THEN EXCLUDED.last_seen_post_id
                    ELSE channel_checkpoints.last_seen_post_id
                END,
                last_seen_post_at = GREATEST(channel_checkpoints.last_seen_post_at, EXCLUDED.last_seen_post_at),
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(channel_id)
        .bind(last_seen_post_id)
        .bind(last_seen_post_at)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn advance_channel_checkpoint(
        &self,
        channel_id: &str,
        last_seen_post_id: &str,
        last_seen_post_at: DateTime<Utc>,
    ) -> Result<(), ThreadBotError> {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE channel_checkpoints
            SET last_seen_post_id = $2,
                last_seen_post_at = GREATEST(last_seen_post_at, $3),
                updated_at = $4
            WHERE channel_id = $1
              AND is_reconciled = true
              AND $3 >= last_seen_post_at
            "#,
        )
        .bind(channel_id)
        .bind(last_seen_post_id)
        .bind(last_seen_post_at)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn set_all_channels_not_reconciled(&self) -> Result<(), ThreadBotError> {
        sqlx::query("UPDATE channel_checkpoints SET is_reconciled = false")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn set_channel_reconciled(&self, channel_id: &str) -> Result<(), ThreadBotError> {
        let now = Utc::now();

        sqlx::query(
            "UPDATE channel_checkpoints SET is_reconciled = true, updated_at = $2 WHERE channel_id = $1",
        )
        .bind(channel_id)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[derive(FromRow)]
struct CheckpointRow {
    channel_id: String,
    last_seen_post_id: String,
    last_seen_post_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    is_reconciled: bool,
}

impl From<CheckpointRow> for ChannelCheckpoint {
    fn from(row: CheckpointRow) -> Self {
        Self {
            channel_id: row.channel_id,
            last_seen_post_id: row.last_seen_post_id,
            last_seen_post_at: row.last_seen_post_at,
            updated_at: row.updated_at,
            is_reconciled: row.is_reconciled,
        }
    }
}
