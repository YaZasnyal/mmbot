//! Shared test harness for thread-bot integration tests.
//!
//! Provides per-test database isolation by creating a fresh PostgreSQL
//! database for each test and dropping it on cleanup.
//!
//! Requires a running PostgreSQL instance. Start with:
//!   just test-env-start

use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use thread_bot::{PgThreadStore, ThreadStatus, UpsertThread, UpsertThreadMessage};
use uuid::Uuid;

/// Returns the base Postgres URL (without database name).
/// Uses TEST_DATABASE_URL env var or falls back to docker-compose test-db defaults.
fn base_pg_url() -> String {
    if let Ok(url) = std::env::var("TEST_DATABASE_URL") {
        // Strip the database name from the URL if present
        // e.g., postgres://user:pass@host:port/dbname -> postgres://user:pass@host:port
        if let Some(pos) = url.rfind('/') {
            // Make sure we're not stripping the scheme part (://)
            if pos > url.find("://").map(|p| p + 2).unwrap_or(0) {
                return url[..pos].to_string();
            }
        }
        url
    } else {
        "postgres://test:test@localhost:5433".to_string()
    }
}

/// Per-test database isolation harness.
///
/// Creates a uniquely-named database on construction and force-drops it
/// on [`cleanup`](Self::cleanup). Every test gets a completely clean schema.
pub struct TestDb {
    pub db_name: String,
    pub pool: PgPool,
    admin_pool: PgPool,
}

impl TestDb {
    /// Create a fresh isolated database for a single test.
    pub async fn new() -> Self {
        let db_name = format!("test_{}", Uuid::new_v4().simple());
        let base_url = base_pg_url();

        // Connect to the default maintenance database
        let admin_url = format!("{}/postgres", base_url);
        let admin_pool = PgPool::connect(&admin_url)
            .await
            .expect("Cannot connect to test Postgres. Is `just test-env-start` running?");

        // Create the test database
        sqlx::query(&format!("CREATE DATABASE \"{}\"", db_name))
            .execute(&admin_pool)
            .await
            .expect("Failed to create test database");

        // Connect to the new test database
        let test_url = format!("{}/{}", base_url, db_name);
        let pool = PgPool::connect(&test_url)
            .await
            .expect("Failed to connect to test database");

        Self {
            db_name,
            pool,
            admin_pool,
        }
    }

    /// Create a PgThreadStore (runs migrations automatically).
    pub async fn store(&self) -> PgThreadStore {
        PgThreadStore::new(self.pool.clone())
            .await
            .expect("Failed to create PgThreadStore")
    }

    /// Drop the test database.
    pub async fn cleanup(self) {
        // Close all connections to the test database first
        self.pool.close().await;

        // Force-drop the database (PG 13+)
        sqlx::query(&format!(
            "DROP DATABASE IF EXISTS \"{}\" WITH (FORCE)",
            self.db_name
        ))
        .execute(&self.admin_pool)
        .await
        .expect("Failed to drop test database");

        self.admin_pool.close().await;
    }
}

/// Create a sample [`UpsertThread`] with predictable field names based on `suffix`.
pub fn make_thread(suffix: &str) -> UpsertThread {
    UpsertThread {
        thread_id: format!("thread_{}", suffix),
        root_post_id: format!("root_post_{}", suffix),
        channel_id: format!("channel_{}", suffix),
        creator_user_id: format!("user_{}", suffix),
        status: ThreadStatus::New,
        metadata: json!({}),
    }
}

/// Create a sample [`UpsertThreadMessage`] belonging to `thread_id`.
pub fn make_message(thread_id: &str, post_suffix: &str) -> UpsertThreadMessage {
    UpsertThreadMessage {
        post_id: format!("post_{}", post_suffix),
        thread_id: thread_id.to_string(),
        user_id: "user_1".to_string(),
        is_bot_message: false,
        root_id: Some(format!("root_post_{}", post_suffix)),
        parent_post_id: None,
        metadata: json!({}),
        post_created_at: Utc::now(),
        post_updated_at: None,
        post_deleted_at: None,
    }
}
