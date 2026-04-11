//! Test utilities for thread-bot unit tests.
//!
//! Provides in-memory mock implementations of [`ThreadStore`] and
//! [`ThreadHandler`], plus helpers for setting up mock Mattermost API
//! responses via wiremock.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::{mpsc, Mutex, RwLock};
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

use mattermost_api::apis::configuration::Configuration;
use mattermost_api::models;

use crate::actor::{ActorCtx, ThreadCommand};
use crate::error::ThreadBotError;
use crate::handler::{ThreadCloseReason, ThreadContext, ThreadEffect, ThreadHandler};
use crate::store::ThreadStore;
use crate::types::*;

// ─── MockStore ──────────────────────────────────────────────────────────────

/// In-memory implementation of [`ThreadStore`] for tests.
pub struct MockStore {
    state: RwLock<MockStoreState>,
}

struct MockStoreState {
    threads: HashMap<String, ThreadRecord>,
    messages: HashMap<String, ThreadMessageRecord>,
    reactions: Vec<StoredReaction>,
}

struct StoredReaction {
    thread_id: String,
    post_id: String,
    user_id: String,
    emoji_name: String,
    action: ReactionAction,
    created_at: DateTime<Utc>,
}

impl MockStore {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(MockStoreState {
                threads: HashMap::new(),
                messages: HashMap::new(),
                reactions: Vec::new(),
            }),
        }
    }

    /// Read a thread record for test assertions.
    pub async fn thread_snapshot(&self, thread_id: &str) -> Option<ThreadRecord> {
        self.state.read().await.threads.get(thread_id).cloned()
    }

    /// Read a message record for test assertions.
    pub async fn message_snapshot(&self, post_id: &str) -> Option<ThreadMessageRecord> {
        self.state.read().await.messages.get(post_id).cloned()
    }
}

#[async_trait]
impl ThreadStore for MockStore {
    async fn upsert_thread(&self, input: UpsertThread) -> Result<ThreadRecord, ThreadBotError> {
        let mut state = self.state.write().await;
        let now = Utc::now();

        let record = if let Some(existing) = state.threads.get(&input.thread_id) {
            ThreadRecord {
                status: input.status,
                metadata: input.metadata,
                updated_at: now,
                ..existing.clone()
            }
        } else {
            ThreadRecord {
                thread_id: input.thread_id.clone(),
                root_post_id: input.root_post_id,
                channel_id: input.channel_id,
                creator_user_id: input.creator_user_id,
                status: input.status,
                metadata: input.metadata,
                last_seen_post_id: None,
                last_seen_post_at: None,
                last_processed_post_id: None,
                last_processed_post_at: None,
                created_at: now,
                updated_at: now,
            }
        };

        state.threads.insert(input.thread_id, record.clone());
        Ok(record)
    }

    async fn get_thread(&self, thread_id: &str) -> Result<Option<ThreadRecord>, ThreadBotError> {
        Ok(self.state.read().await.threads.get(thread_id).cloned())
    }

    async fn get_thread_by_post(
        &self,
        post_id: &str,
    ) -> Result<Option<ThreadRecord>, ThreadBotError> {
        let state = self.state.read().await;

        // Check root posts first
        if let Some(thread) = state.threads.get(post_id) {
            return Ok(Some(thread.clone()));
        }

        // Then check messages
        if let Some(msg) = state.messages.get(post_id) {
            return Ok(state.threads.get(&msg.thread_id).cloned());
        }

        Ok(None)
    }

    async fn list_threads_by_status(
        &self,
        statuses: &[ThreadStatus],
    ) -> Result<Vec<ThreadRecord>, ThreadBotError> {
        let state = self.state.read().await;
        Ok(state
            .threads
            .values()
            .filter(|t| statuses.contains(&t.status))
            .cloned()
            .collect())
    }

    async fn update_thread_status(
        &self,
        thread_id: &str,
        status: ThreadStatus,
    ) -> Result<(), ThreadBotError> {
        let mut state = self.state.write().await;
        match state.threads.get_mut(thread_id) {
            Some(thread) => {
                thread.status = status;
                thread.updated_at = Utc::now();
                Ok(())
            }
            None => Err(ThreadBotError::ThreadNotFound(thread_id.to_string())),
        }
    }

    async fn set_thread_metadata(
        &self,
        thread_id: &str,
        metadata: serde_json::Value,
    ) -> Result<(), ThreadBotError> {
        let mut state = self.state.write().await;
        match state.threads.get_mut(thread_id) {
            Some(thread) => {
                thread.metadata = metadata;
                thread.updated_at = Utc::now();
                Ok(())
            }
            None => Err(ThreadBotError::ThreadNotFound(thread_id.to_string())),
        }
    }

    async fn update_thread_seen(
        &self,
        thread_id: &str,
        post_id: &str,
        seen_at: DateTime<Utc>,
    ) -> Result<(), ThreadBotError> {
        let mut state = self.state.write().await;
        match state.threads.get_mut(thread_id) {
            Some(thread) => {
                thread.last_seen_post_id = Some(post_id.to_string());
                thread.last_seen_post_at = Some(seen_at);
                thread.updated_at = Utc::now();
                Ok(())
            }
            None => Err(ThreadBotError::ThreadNotFound(thread_id.to_string())),
        }
    }

    async fn update_thread_processed(
        &self,
        thread_id: &str,
        post_id: &str,
        processed_at: DateTime<Utc>,
    ) -> Result<(), ThreadBotError> {
        let mut state = self.state.write().await;
        match state.threads.get_mut(thread_id) {
            Some(thread) => {
                thread.last_processed_post_id = Some(post_id.to_string());
                thread.last_processed_post_at = Some(processed_at);
                thread.updated_at = Utc::now();
                Ok(())
            }
            None => Err(ThreadBotError::ThreadNotFound(thread_id.to_string())),
        }
    }

    async fn upsert_message(
        &self,
        input: UpsertThreadMessage,
    ) -> Result<ThreadMessageRecord, ThreadBotError> {
        let mut state = self.state.write().await;
        let now = Utc::now();

        let record = ThreadMessageRecord {
            post_id: input.post_id.clone(),
            thread_id: input.thread_id,
            user_id: input.user_id,
            is_bot_message: input.is_bot_message,
            root_id: input.root_id,
            parent_post_id: input.parent_post_id,
            metadata: input.metadata,
            post_created_at: input.post_created_at,
            post_updated_at: input.post_updated_at,
            post_deleted_at: input.post_deleted_at,
            created_at: now,
            updated_at: now,
        };

        state.messages.insert(input.post_id, record.clone());
        Ok(record)
    }

    async fn get_message(
        &self,
        post_id: &str,
    ) -> Result<Option<ThreadMessageRecord>, ThreadBotError> {
        Ok(self.state.read().await.messages.get(post_id).cloned())
    }

    async fn list_thread_messages(
        &self,
        thread_id: &str,
    ) -> Result<Vec<ThreadMessageRecord>, ThreadBotError> {
        let state = self.state.read().await;
        let mut msgs: Vec<_> = state
            .messages
            .values()
            .filter(|m| m.thread_id == thread_id)
            .cloned()
            .collect();
        msgs.sort_by_key(|m| m.post_created_at);
        Ok(msgs)
    }

    async fn set_message_metadata(
        &self,
        post_id: &str,
        metadata: serde_json::Value,
    ) -> Result<(), ThreadBotError> {
        let mut state = self.state.write().await;
        match state.messages.get_mut(post_id) {
            Some(msg) => {
                msg.metadata = metadata;
                msg.updated_at = Utc::now();
                Ok(())
            }
            None => Err(ThreadBotError::MessageNotFound(post_id.to_string())),
        }
    }

    async fn append_reaction(&self, input: AppendReaction) -> Result<(), ThreadBotError> {
        let mut state = self.state.write().await;

        let thread_id = match input.thread_id {
            Some(id) => id,
            None => {
                if let Some(thread) = state.threads.get(&input.post_id) {
                    thread.thread_id.clone()
                } else if let Some(msg) = state.messages.get(&input.post_id) {
                    msg.thread_id.clone()
                } else {
                    return Ok(());
                }
            }
        };

        state.reactions.push(StoredReaction {
            thread_id,
            post_id: input.post_id,
            user_id: input.user_id,
            emoji_name: input.emoji_name,
            action: input.action,
            created_at: Utc::now(),
        });

        Ok(())
    }

    async fn list_thread_reactions(
        &self,
        thread_id: &str,
    ) -> Result<Vec<ThreadReaction>, ThreadBotError> {
        let state = self.state.read().await;
        Ok(state
            .reactions
            .iter()
            .filter(|r| r.thread_id == thread_id)
            .map(|r| ThreadReaction {
                post_id: r.post_id.clone(),
                user_id: r.user_id.clone(),
                emoji_name: r.emoji_name.clone(),
                action: r.action,
                created_at: r.created_at,
                is_new: false,
            })
            .collect())
    }
}

// ─── MockHandler ────────────────────────────────────────────────────────────

/// Configurable mock implementation of [`ThreadHandler`] for tests.
///
/// Tracks call counts and provides channels for synchronizing with the
/// actor under test.
pub struct MockHandler {
    effects_queue: Mutex<VecDeque<Vec<ThreadEffect>>>,
    default_effects: Vec<ThreadEffect>,
    handle_delay: Option<Duration>,
    should_error: AtomicBool,

    /// Sends a signal each time `handle()` is entered.
    handle_entered_tx: mpsc::UnboundedSender<()>,
    /// Receive signals when `handle()` is entered.
    pub handle_entered_rx: Mutex<mpsc::UnboundedReceiver<()>>,

    /// Number of times `handle()` was called.
    pub handle_call_count: AtomicUsize,
    /// Reasons passed to `on_thread_closed()`.
    pub closed_reasons: Mutex<Vec<ThreadCloseReason>>,
    /// Whether `should_track()` returns true.
    should_track_result: bool,
}

impl MockHandler {
    /// Create a handler that returns `Noop` by default.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            effects_queue: Mutex::new(VecDeque::new()),
            default_effects: vec![ThreadEffect::Noop],
            handle_delay: None,
            should_error: AtomicBool::new(false),
            handle_entered_tx: tx,
            handle_entered_rx: Mutex::new(rx),
            handle_call_count: AtomicUsize::new(0),
            closed_reasons: Mutex::new(Vec::new()),
            should_track_result: true,
        }
    }

    /// Set the default effects returned by `handle()`.
    pub fn with_default_effects(mut self, effects: Vec<ThreadEffect>) -> Self {
        self.default_effects = effects;
        self
    }

    /// Queue specific effects for successive calls (FIFO).
    /// After the queue is exhausted, `default_effects` are used.
    pub fn with_effects_queue(mut self, queue: Vec<Vec<ThreadEffect>>) -> Self {
        self.effects_queue = Mutex::new(queue.into());
        self
    }

    /// Set delay before `handle()` returns (for testing interruption).
    pub fn with_handle_delay(mut self, delay: Duration) -> Self {
        self.handle_delay = Some(delay);
        self
    }

    /// Make `handle()` return an error.
    pub fn with_error(self) -> Self {
        self.should_error.store(true, Ordering::SeqCst);
        self
    }

    /// Set whether `should_track()` returns true.
    pub fn with_should_track(mut self, value: bool) -> Self {
        self.should_track_result = value;
        self
    }

    /// Get the number of times `handle()` was called.
    pub fn call_count(&self) -> usize {
        self.handle_call_count.load(Ordering::SeqCst)
    }

    /// Wait for `handle()` to be entered.
    pub async fn wait_handle_entered(&self) {
        self.handle_entered_rx.lock().await.recv().await;
    }
}

#[async_trait]
impl ThreadHandler for MockHandler {
    fn id(&self) -> &'static str {
        "mock-handler"
    }

    async fn should_track(
        &self,
        _thread: &Thread,
        _ctx: &ThreadContext,
    ) -> Result<bool, ThreadBotError> {
        Ok(self.should_track_result)
    }

    async fn handle(
        &self,
        _thread: &Thread,
        _ctx: &ThreadContext,
    ) -> Result<Vec<ThreadEffect>, ThreadBotError> {
        self.handle_call_count.fetch_add(1, Ordering::SeqCst);
        let _ = self.handle_entered_tx.send(());

        if let Some(delay) = self.handle_delay {
            tokio::time::sleep(delay).await;
        }

        if self.should_error.load(Ordering::SeqCst) {
            return Err(ThreadBotError::Internal(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "mock handler error",
            ))));
        }

        let mut queue = self.effects_queue.lock().await;
        if let Some(effects) = queue.pop_front() {
            Ok(effects)
        } else {
            Ok(self.default_effects.clone())
        }
    }

    async fn on_thread_closed(
        &self,
        _thread: &Thread,
        reason: ThreadCloseReason,
        _ctx: &ThreadContext,
    ) -> Result<(), ThreadBotError> {
        self.closed_reasons.lock().await.push(reason);
        Ok(())
    }
}

// ─── Mattermost API mock helpers ────────────────────────────────────────────

/// Start a wiremock server with Mattermost API endpoints.
///
/// Returns the server (keep alive!) and a `Configuration` pointing to it.
pub async fn setup_mm_mock(posts: Vec<models::Post>) -> (MockServer, Arc<Configuration>) {
    let server = MockServer::start().await;

    // Build PostList JSON
    let mut post_map = HashMap::new();
    let mut order = Vec::new();
    for post in &posts {
        order.push(post.id.clone());
        post_map.insert(post.id.clone(), post.clone());
    }

    let post_list = serde_json::json!({
        "order": order,
        "posts": post_map,
        "next_post_id": "",
        "prev_post_id": "",
        "has_next": false,
    });

    // GET /api/v4/posts/{id}/thread
    Mock::given(method("GET"))
        .and(path_regex(r"/api/v4/posts/.+/thread"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&post_list))
        .mount(&server)
        .await;

    // POST /api/v4/posts  (Reply effect)
    Mock::given(method("POST"))
        .and(path_regex(r"/api/v4/posts$"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "id": "reply_post_id",
            "create_at": Utc::now().timestamp_millis(),
            "message": "mock reply",
        })))
        .mount(&server)
        .await;

    // PUT /api/v4/posts/{id}/patch  (UpdateMessage effect)
    Mock::given(method("PUT"))
        .and(path_regex(r"/api/v4/posts/.+/patch"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "patched_post_id",
            "message": "updated",
        })))
        .mount(&server)
        .await;

    let mut config = Configuration::default();
    config.base_path = server.uri();

    (server, Arc::new(config))
}

/// Create a Mattermost [`Post`](models::Post) for tests.
pub fn make_mm_post(id: &str, user_id: &str, message: &str, root_id: Option<&str>) -> models::Post {
    models::Post {
        id: id.to_string(),
        user_id: Some(user_id.to_string()),
        message: Some(message.to_string()),
        channel_id: Some("test_channel".to_string()),
        root_id: root_id.map(|s| s.to_string()),
        create_at: Some(Utc::now().timestamp_millis()),
        update_at: Some(Utc::now().timestamp_millis()),
        ..Default::default()
    }
}

/// Spawn a test actor with all dependencies wired up.
///
/// Creates a thread record in the mock store, starts the actor task,
/// and returns handles for driving and inspecting the test.
pub async fn spawn_test_actor(
    handler: MockHandler,
    thread_id: &str,
    posts: Vec<models::Post>,
    debounce: Duration,
) -> (
    mpsc::Sender<ThreadCommand>,
    Arc<MockStore>,
    Arc<MockHandler>,
    MockServer,
) {
    let (server, mm_config) = setup_mm_mock(posts).await;
    let store = Arc::new(MockStore::new());
    let handler = Arc::new(handler);

    // Seed the thread record
    store
        .upsert_thread(UpsertThread {
            thread_id: thread_id.to_string(),
            root_post_id: thread_id.to_string(),
            channel_id: "test_channel".to_string(),
            creator_user_id: "user_1".to_string(),
            status: ThreadStatus::New,
            metadata: serde_json::Value::Null,
        })
        .await
        .unwrap();

    let ctx = ThreadContext {
        config: Arc::clone(&mm_config),
        store: Arc::clone(&store) as Arc<dyn ThreadStore>,
        plugin_id: "mock-handler",
    };

    let (tx, rx) = mpsc::channel(64);

    let actor_ctx = ActorCtx {
        handler: Arc::clone(&handler),
        store: Arc::clone(&store) as Arc<dyn ThreadStore>,
        ctx,
        mm_config,
        debounce,
        thread_id: thread_id.to_string(),
        bot_user_id: Arc::new(RwLock::new(Some("bot_user".to_string()))),
    };

    tokio::spawn(crate::actor::thread_actor(rx, actor_ctx));

    (tx, store, handler, server)
}
