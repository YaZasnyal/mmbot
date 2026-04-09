use crate::error::ThreadBotError;
use crate::store::ThreadStore;
use crate::types::{
    AppendReaction, ReactionAction, ThreadMessageRecord, ThreadReaction, ThreadRecord,
    ThreadStatus, UpsertThread, UpsertThreadMessage,
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

// Helper: parse ThreadStatus from DB string
fn parse_status(s: &str) -> ThreadStatus {
    match s {
        "new" => ThreadStatus::New,
        "active" => ThreadStatus::Active,
        "resolved" => ThreadStatus::Resolved,
        "stopped" => ThreadStatus::Stopped,
        _ => ThreadStatus::Active,
    }
}

// Helper: ThreadStatus to DB string
fn status_str(status: &ThreadStatus) -> &'static str {
    match status {
        ThreadStatus::New => "new",
        ThreadStatus::Active => "active",
        ThreadStatus::Resolved => "resolved",
        ThreadStatus::Stopped => "stopped",
    }
}

// Helper: parse ReactionAction from DB string
fn parse_reaction_action(s: &str) -> ReactionAction {
    match s {
        "added" => ReactionAction::Added,
        "removed" => ReactionAction::Removed,
        _ => ReactionAction::Added,
    }
}

// Helper: ReactionAction to DB string
fn reaction_action_str(action: &ReactionAction) -> &'static str {
    match action {
        ReactionAction::Added => "added",
        ReactionAction::Removed => "removed",
    }
}

// Row types for sqlx queries
#[derive(FromRow)]
struct ThreadRow {
    thread_id: String,
    root_post_id: String,
    channel_id: String,
    creator_user_id: String,
    status: String,
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
            status: parse_status(&row.status),
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

#[derive(FromRow)]
struct ReactionRow {
    post_id: String,
    user_id: String,
    emoji_name: String,
    action: String,
    created_at: DateTime<Utc>,
}

impl From<ReactionRow> for ThreadReaction {
    fn from(row: ReactionRow) -> Self {
        Self {
            post_id: row.post_id,
            user_id: row.user_id,
            emoji_name: row.emoji_name,
            action: parse_reaction_action(&row.action),
            created_at: row.created_at,
        }
    }
}

const THREAD_COLUMNS: &str = r#"
    thread_id, root_post_id, channel_id, creator_user_id, status,
    metadata, last_seen_post_id, last_seen_post_at,
    last_processed_post_id, last_processed_post_at,
    created_at, updated_at
"#;

const MESSAGE_COLUMNS: &str = r#"
    post_id, thread_id, user_id, is_bot_message, root_id, parent_post_id,
    metadata, post_created_at, post_updated_at, post_deleted_at,
    created_at, updated_at
"#;

#[async_trait]
impl ThreadStore for PgThreadStore {
    async fn upsert_thread(&self, input: UpsertThread) -> Result<ThreadRecord, ThreadBotError> {
        let now = Utc::now();
        let status = status_str(&input.status);

        let sql = format!(
            r#"
            INSERT INTO threads (thread_id, root_post_id, channel_id, creator_user_id, status, metadata, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
            ON CONFLICT (thread_id) DO UPDATE SET
                status = EXCLUDED.status,
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
            .bind(status)
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

    async fn list_threads_by_status(
        &self,
        statuses: &[ThreadStatus],
    ) -> Result<Vec<ThreadRecord>, ThreadBotError> {
        let status_strings: Vec<&str> = statuses.iter().map(status_str).collect();

        let sql = format!(
            "SELECT {THREAD_COLUMNS} FROM threads WHERE status = ANY($1) ORDER BY updated_at DESC"
        );

        let rows: Vec<ThreadRow> = sqlx::query_as(&sql)
            .bind(&status_strings)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update_thread_status(
        &self,
        thread_id: &str,
        status: ThreadStatus,
    ) -> Result<(), ThreadBotError> {
        let status = status_str(&status);
        let now = Utc::now();

        let result =
            sqlx::query("UPDATE threads SET status = $2, updated_at = $3 WHERE thread_id = $1")
                .bind(thread_id)
                .bind(status)
                .bind(now)
                .execute(&self.pool)
                .await?;

        if result.rows_affected() == 0 {
            return Err(ThreadBotError::ThreadNotFound(thread_id.to_string()));
        }
        Ok(())
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

    async fn append_reaction(&self, input: AppendReaction) -> Result<(), ThreadBotError> {
        let action = reaction_action_str(&input.action);
        let now = Utc::now();

        let thread_id = match input.thread_id {
            Some(id) => id,
            None => {
                let record = self.get_thread_by_post(&input.post_id).await?;
                match record {
                    Some(r) => r.thread_id,
                    None => return Ok(()), // Not a tracked thread, skip
                }
            }
        };

        sqlx::query(
            r#"
            INSERT INTO thread_reactions (thread_id, post_id, user_id, emoji_name, action, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&thread_id)
        .bind(&input.post_id)
        .bind(&input.user_id)
        .bind(&input.emoji_name)
        .bind(action)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn list_thread_reactions(
        &self,
        thread_id: &str,
    ) -> Result<Vec<ThreadReaction>, ThreadBotError> {
        let rows: Vec<ReactionRow> = sqlx::query_as(
            r#"
            SELECT post_id, user_id, emoji_name, action, created_at
            FROM thread_reactions
            WHERE thread_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(thread_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}
