//! Integration tests for PgThreadStore
//!
//! Requires a running PostgreSQL instance. Start with:
//!   just test-env-start
//!
//! Each test creates an isolated database and drops it after completion.

mod common;

use chrono::Utc;
use common::{make_message, make_thread, TestDb};
use serde_json::json;
use thread_bot::{
    AppendReaction, ReactionAction, ThreadBotError, ThreadStatus, ThreadStore, UpsertThread,
    UpsertThreadMessage,
};

// ---------------------------------------------------------------------------
// Thread CRUD tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn upsert_thread_creates_new() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let input = make_thread("1");
    let record = store.upsert_thread(input.clone()).await.unwrap();

    assert_eq!(record.thread_id, "thread_1");
    assert_eq!(record.root_post_id, "root_post_1");
    assert_eq!(record.channel_id, "channel_1");
    assert_eq!(record.creator_user_id, "user_1");
    assert_eq!(record.status, ThreadStatus::New);
    assert_eq!(record.metadata, json!({}));
    assert!(record.last_seen_post_id.is_none());
    assert!(record.last_processed_post_id.is_none());

    db.cleanup().await;
}

#[tokio::test]
async fn upsert_thread_updates_existing() {
    let db = TestDb::new().await;
    let store = db.store().await;

    // Create thread
    let input = make_thread("1");
    let created = store.upsert_thread(input).await.unwrap();

    // Upsert with changed status and metadata
    let updated_input = UpsertThread {
        thread_id: "thread_1".to_string(),
        root_post_id: "root_post_1".to_string(),
        channel_id: "channel_1".to_string(),
        creator_user_id: "user_1".to_string(),
        status: ThreadStatus::Active,
        metadata: json!({"key": "value"}),
    };
    let updated = store.upsert_thread(updated_input).await.unwrap();

    assert_eq!(updated.thread_id, "thread_1");
    assert_eq!(updated.status, ThreadStatus::Active);
    assert_eq!(updated.metadata, json!({"key": "value"}));
    // created_at should not change
    assert_eq!(updated.created_at, created.created_at);
    // updated_at should change
    assert!(updated.updated_at >= created.updated_at);

    db.cleanup().await;
}

#[tokio::test]
async fn get_thread_existing() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let input = make_thread("1");
    store.upsert_thread(input).await.unwrap();

    let found = store.get_thread("thread_1").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().thread_id, "thread_1");

    db.cleanup().await;
}

#[tokio::test]
async fn get_thread_not_found() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let found = store.get_thread("nonexistent").await.unwrap();
    assert!(found.is_none());

    db.cleanup().await;
}

#[tokio::test]
async fn get_thread_by_root_post_id() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let input = make_thread("1");
    store.upsert_thread(input).await.unwrap();

    // Should find thread by its root_post_id
    let found = store.get_thread_by_post("root_post_1").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().thread_id, "thread_1");

    db.cleanup().await;
}

#[tokio::test]
async fn get_thread_by_message_post_id() {
    let db = TestDb::new().await;
    let store = db.store().await;

    // Create thread and a message in it
    let input = make_thread("1");
    store.upsert_thread(input).await.unwrap();

    let msg = UpsertThreadMessage {
        post_id: "reply_42".to_string(),
        thread_id: "thread_1".to_string(),
        user_id: "user_2".to_string(),
        is_bot_message: false,
        root_id: Some("root_post_1".to_string()),
        parent_post_id: None,
        metadata: json!({}),
        post_created_at: Utc::now(),
        post_updated_at: None,
        post_deleted_at: None,
    };
    store.upsert_message(msg).await.unwrap();

    // Should find thread by the message's post_id
    let found = store.get_thread_by_post("reply_42").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().thread_id, "thread_1");

    db.cleanup().await;
}

#[tokio::test]
async fn get_thread_by_post_not_found() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let found = store.get_thread_by_post("unknown_post").await.unwrap();
    assert!(found.is_none());

    db.cleanup().await;
}

#[tokio::test]
async fn list_threads_by_status_filters_correctly() {
    let db = TestDb::new().await;
    let store = db.store().await;

    // Create threads with different statuses
    let mut t1 = make_thread("1");
    t1.status = ThreadStatus::New;
    store.upsert_thread(t1).await.unwrap();

    let mut t2 = make_thread("2");
    t2.status = ThreadStatus::Active;
    store.upsert_thread(t2).await.unwrap();

    let mut t3 = make_thread("3");
    t3.status = ThreadStatus::Resolved;
    store.upsert_thread(t3).await.unwrap();

    let mut t4 = make_thread("4");
    t4.status = ThreadStatus::Stopped;
    store.upsert_thread(t4).await.unwrap();

    // Filter by single status
    let active = store
        .list_threads_by_status(&[ThreadStatus::Active], None, None)
        .await
        .unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].thread_id, "thread_2");

    // Filter by multiple statuses
    let open = store
        .list_threads_by_status(&[ThreadStatus::New, ThreadStatus::Active], None, None)
        .await
        .unwrap();
    assert_eq!(open.len(), 2);

    // Empty filter returns nothing
    let empty = store.list_threads_by_status(&[], None, None).await.unwrap();
    assert_eq!(empty.len(), 0);

    db.cleanup().await;
}

#[tokio::test]
async fn update_thread_status_success() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let input = make_thread("1");
    store.upsert_thread(input).await.unwrap();

    store
        .update_thread_status("thread_1", ThreadStatus::Active)
        .await
        .unwrap();

    let found = store.get_thread("thread_1").await.unwrap().unwrap();
    assert_eq!(found.status, ThreadStatus::Active);

    db.cleanup().await;
}

#[tokio::test]
async fn update_thread_status_not_found() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let result = store
        .update_thread_status("nonexistent", ThreadStatus::Active)
        .await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ThreadBotError::ThreadNotFound(_)
    ));

    db.cleanup().await;
}

#[tokio::test]
async fn set_thread_metadata_success() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let input = make_thread("1");
    store.upsert_thread(input).await.unwrap();

    let meta = json!({"llm_model": "gpt-4", "tokens_used": 1500});
    store
        .set_thread_metadata("thread_1", meta.clone())
        .await
        .unwrap();

    let found = store.get_thread("thread_1").await.unwrap().unwrap();
    assert_eq!(found.metadata, meta);

    db.cleanup().await;
}

#[tokio::test]
async fn set_thread_metadata_not_found() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let result = store.set_thread_metadata("nonexistent", json!({})).await;

    assert!(matches!(
        result.unwrap_err(),
        ThreadBotError::ThreadNotFound(_)
    ));

    db.cleanup().await;
}

#[tokio::test]
async fn update_thread_seen_and_processed() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let input = make_thread("1");
    store.upsert_thread(input).await.unwrap();

    let now = Utc::now();

    // Update seen
    store
        .update_thread_seen("thread_1", "post_seen_1", now)
        .await
        .unwrap();

    let found = store.get_thread("thread_1").await.unwrap().unwrap();
    assert_eq!(found.last_seen_post_id.as_deref(), Some("post_seen_1"));
    assert!(found.last_seen_post_at.is_some());

    // Update processed
    store
        .update_thread_processed("thread_1", "post_proc_1", now)
        .await
        .unwrap();

    let found = store.get_thread("thread_1").await.unwrap().unwrap();
    assert_eq!(found.last_processed_post_id.as_deref(), Some("post_proc_1"));
    assert!(found.last_processed_post_at.is_some());

    db.cleanup().await;
}

#[tokio::test]
async fn update_thread_seen_not_found() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let result = store
        .update_thread_seen("nonexistent", "post_1", Utc::now())
        .await;

    assert!(matches!(
        result.unwrap_err(),
        ThreadBotError::ThreadNotFound(_)
    ));

    db.cleanup().await;
}

#[tokio::test]
async fn update_thread_processed_not_found() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let result = store
        .update_thread_processed("nonexistent", "post_1", Utc::now())
        .await;

    assert!(matches!(
        result.unwrap_err(),
        ThreadBotError::ThreadNotFound(_)
    ));

    db.cleanup().await;
}

// ---------------------------------------------------------------------------
// Message CRUD tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn upsert_message_creates_new() {
    let db = TestDb::new().await;
    let store = db.store().await;

    // Need a thread first (FK constraint)
    let thread = make_thread("1");
    store.upsert_thread(thread).await.unwrap();

    let msg = make_message("thread_1", "msg1");
    let record = store.upsert_message(msg).await.unwrap();

    assert_eq!(record.post_id, "post_msg1");
    assert_eq!(record.thread_id, "thread_1");
    assert_eq!(record.user_id, "user_1");
    assert!(!record.is_bot_message);

    db.cleanup().await;
}

#[tokio::test]
async fn upsert_message_updates_existing() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let thread = make_thread("1");
    store.upsert_thread(thread).await.unwrap();

    // Create message
    let msg = make_message("thread_1", "msg1");
    let created = store.upsert_message(msg).await.unwrap();

    // Upsert same post_id with changes
    let updated_msg = UpsertThreadMessage {
        post_id: "post_msg1".to_string(),
        thread_id: "thread_1".to_string(),
        user_id: "user_1".to_string(),
        is_bot_message: true, // changed
        root_id: None,
        parent_post_id: None,
        metadata: json!({"edited": true}), // changed
        post_created_at: created.post_created_at,
        post_updated_at: Some(Utc::now()),
        post_deleted_at: None,
    };
    let updated = store.upsert_message(updated_msg).await.unwrap();

    assert_eq!(updated.post_id, "post_msg1");
    assert!(updated.is_bot_message);
    assert_eq!(updated.metadata, json!({"edited": true}));
    assert!(updated.post_updated_at.is_some());

    db.cleanup().await;
}

#[tokio::test]
async fn get_message_existing() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let thread = make_thread("1");
    store.upsert_thread(thread).await.unwrap();

    let msg = make_message("thread_1", "msg1");
    store.upsert_message(msg).await.unwrap();

    let found = store.get_message("post_msg1").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().post_id, "post_msg1");

    db.cleanup().await;
}

#[tokio::test]
async fn get_message_not_found() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let found = store.get_message("nonexistent").await.unwrap();
    assert!(found.is_none());

    db.cleanup().await;
}

#[tokio::test]
async fn list_thread_messages_ordered_by_created_at() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let thread = make_thread("1");
    store.upsert_thread(thread).await.unwrap();

    // Insert messages with explicit timestamps to ensure ordering
    let base = Utc::now();
    for i in 0..5 {
        let msg = UpsertThreadMessage {
            post_id: format!("post_{}", i),
            thread_id: "thread_1".to_string(),
            user_id: "user_1".to_string(),
            is_bot_message: i % 2 == 0,
            root_id: None,
            parent_post_id: None,
            metadata: json!({}),
            post_created_at: base + chrono::Duration::seconds(i),
            post_updated_at: None,
            post_deleted_at: None,
        };
        store.upsert_message(msg).await.unwrap();
    }

    let messages = store.list_thread_messages("thread_1").await.unwrap();
    assert_eq!(messages.len(), 5);

    // Verify ascending order by post_created_at
    for i in 1..messages.len() {
        assert!(messages[i].post_created_at >= messages[i - 1].post_created_at);
    }

    // First is post_0, last is post_4
    assert_eq!(messages[0].post_id, "post_0");
    assert_eq!(messages[4].post_id, "post_4");

    db.cleanup().await;
}

#[tokio::test]
async fn list_thread_messages_empty() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let thread = make_thread("1");
    store.upsert_thread(thread).await.unwrap();

    let messages = store.list_thread_messages("thread_1").await.unwrap();
    assert!(messages.is_empty());

    db.cleanup().await;
}

#[tokio::test]
async fn set_message_metadata_success() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let thread = make_thread("1");
    store.upsert_thread(thread).await.unwrap();

    let msg = make_message("thread_1", "msg1");
    store.upsert_message(msg).await.unwrap();

    let meta = json!({"llm_response": "Hello!", "confidence": 0.95});
    store
        .set_message_metadata("post_msg1", meta.clone())
        .await
        .unwrap();

    let found = store.get_message("post_msg1").await.unwrap().unwrap();
    assert_eq!(found.metadata, meta);

    db.cleanup().await;
}

#[tokio::test]
async fn set_message_metadata_not_found() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let result = store.set_message_metadata("nonexistent", json!({})).await;

    assert!(matches!(
        result.unwrap_err(),
        ThreadBotError::MessageNotFound(_)
    ));

    db.cleanup().await;
}

// ---------------------------------------------------------------------------
// Reaction tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn append_reaction_with_explicit_thread_id() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let thread = make_thread("1");
    store.upsert_thread(thread).await.unwrap();

    let reaction = AppendReaction {
        thread_id: Some("thread_1".to_string()),
        post_id: "root_post_1".to_string(),
        user_id: "user_2".to_string(),
        emoji_name: "white_check_mark".to_string(),
        action: ReactionAction::Added,
    };
    store.append_reaction(reaction).await.unwrap();

    let reactions = store.list_thread_reactions("thread_1").await.unwrap();
    assert_eq!(reactions.len(), 1);
    assert_eq!(reactions[0].emoji_name, "white_check_mark");
    assert_eq!(reactions[0].user_id, "user_2");
    assert_eq!(reactions[0].action, ReactionAction::Added);

    db.cleanup().await;
}

#[tokio::test]
async fn append_reaction_auto_resolves_thread_by_root_post() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let thread = make_thread("1");
    store.upsert_thread(thread).await.unwrap();

    // No explicit thread_id — should resolve via get_thread_by_post
    let reaction = AppendReaction {
        thread_id: None,
        post_id: "root_post_1".to_string(),
        user_id: "user_2".to_string(),
        emoji_name: "thumbsup".to_string(),
        action: ReactionAction::Added,
    };
    store.append_reaction(reaction).await.unwrap();

    let reactions = store.list_thread_reactions("thread_1").await.unwrap();
    assert_eq!(reactions.len(), 1);
    assert_eq!(reactions[0].emoji_name, "thumbsup");

    db.cleanup().await;
}

#[tokio::test]
async fn append_reaction_auto_resolves_thread_by_message_post() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let thread = make_thread("1");
    store.upsert_thread(thread).await.unwrap();

    // Add a message to the thread
    let msg = UpsertThreadMessage {
        post_id: "reply_99".to_string(),
        thread_id: "thread_1".to_string(),
        user_id: "user_2".to_string(),
        is_bot_message: false,
        root_id: Some("root_post_1".to_string()),
        parent_post_id: None,
        metadata: json!({}),
        post_created_at: Utc::now(),
        post_updated_at: None,
        post_deleted_at: None,
    };
    store.upsert_message(msg).await.unwrap();

    // React to the reply (no explicit thread_id)
    let reaction = AppendReaction {
        thread_id: None,
        post_id: "reply_99".to_string(),
        user_id: "user_3".to_string(),
        emoji_name: "thumbsdown".to_string(),
        action: ReactionAction::Added,
    };
    store.append_reaction(reaction).await.unwrap();

    let reactions = store.list_thread_reactions("thread_1").await.unwrap();
    assert_eq!(reactions.len(), 1);
    assert_eq!(reactions[0].post_id, "reply_99");

    db.cleanup().await;
}

#[tokio::test]
async fn append_reaction_untracked_post_is_noop() {
    let db = TestDb::new().await;
    let store = db.store().await;

    // No thread exists — reaction on untracked post should silently succeed
    let reaction = AppendReaction {
        thread_id: None,
        post_id: "unknown_post".to_string(),
        user_id: "user_1".to_string(),
        emoji_name: "smile".to_string(),
        action: ReactionAction::Added,
    };
    let result = store.append_reaction(reaction).await;
    assert!(result.is_ok());

    db.cleanup().await;
}

#[tokio::test]
async fn list_thread_reactions_ordered_by_created_at() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let thread = make_thread("1");
    store.upsert_thread(thread).await.unwrap();

    let emojis = ["thumbsup", "eyes", "white_check_mark"];
    for emoji in &emojis {
        let reaction = AppendReaction {
            thread_id: Some("thread_1".to_string()),
            post_id: "root_post_1".to_string(),
            user_id: "user_2".to_string(),
            emoji_name: emoji.to_string(),
            action: ReactionAction::Added,
        };
        store.append_reaction(reaction).await.unwrap();
    }

    let reactions = store.list_thread_reactions("thread_1").await.unwrap();
    assert_eq!(reactions.len(), 3);

    // Verify ascending order
    for i in 1..reactions.len() {
        assert!(reactions[i].created_at >= reactions[i - 1].created_at);
    }

    db.cleanup().await;
}

#[tokio::test]
async fn list_thread_reactions_empty() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let thread = make_thread("1");
    store.upsert_thread(thread).await.unwrap();

    let reactions = store.list_thread_reactions("thread_1").await.unwrap();
    assert!(reactions.is_empty());

    db.cleanup().await;
}

#[tokio::test]
async fn reaction_add_and_remove_are_both_recorded() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let thread = make_thread("1");
    store.upsert_thread(thread).await.unwrap();

    // Add a reaction
    store
        .append_reaction(AppendReaction {
            thread_id: Some("thread_1".to_string()),
            post_id: "root_post_1".to_string(),
            user_id: "user_2".to_string(),
            emoji_name: "thumbsup".to_string(),
            action: ReactionAction::Added,
        })
        .await
        .unwrap();

    // Remove the same reaction
    store
        .append_reaction(AppendReaction {
            thread_id: Some("thread_1".to_string()),
            post_id: "root_post_1".to_string(),
            user_id: "user_2".to_string(),
            emoji_name: "thumbsup".to_string(),
            action: ReactionAction::Removed,
        })
        .await
        .unwrap();

    let reactions = store.list_thread_reactions("thread_1").await.unwrap();
    // Both events are stored (append-only log)
    assert_eq!(reactions.len(), 2);
    assert_eq!(reactions[0].action, ReactionAction::Added);
    assert_eq!(reactions[1].action, ReactionAction::Removed);

    db.cleanup().await;
}

// ---------------------------------------------------------------------------
// Cross-feature / integration scenarios
// ---------------------------------------------------------------------------

#[tokio::test]
async fn full_thread_lifecycle() {
    let db = TestDb::new().await;
    let store = db.store().await;

    // 1. Create thread (new)
    let thread = make_thread("lifecycle");
    let record = store.upsert_thread(thread).await.unwrap();
    assert_eq!(record.status, ThreadStatus::New);

    // 2. Activate
    store
        .update_thread_status("thread_lifecycle", ThreadStatus::Active)
        .await
        .unwrap();

    // 3. Add root message
    let root_msg = UpsertThreadMessage {
        post_id: "root_post_lifecycle".to_string(),
        thread_id: "thread_lifecycle".to_string(),
        user_id: "user_lifecycle".to_string(),
        is_bot_message: false,
        root_id: None,
        parent_post_id: None,
        metadata: json!({}),
        post_created_at: Utc::now(),
        post_updated_at: None,
        post_deleted_at: None,
    };
    store.upsert_message(root_msg).await.unwrap();

    // 4. Bot replies
    let bot_msg = UpsertThreadMessage {
        post_id: "bot_reply_1".to_string(),
        thread_id: "thread_lifecycle".to_string(),
        user_id: "bot_user".to_string(),
        is_bot_message: true,
        root_id: Some("root_post_lifecycle".to_string()),
        parent_post_id: None,
        metadata: json!({"llm_model": "gpt-4"}),
        post_created_at: Utc::now(),
        post_updated_at: None,
        post_deleted_at: None,
    };
    store.upsert_message(bot_msg).await.unwrap();

    // 5. Update seen / processed
    let now = Utc::now();
    store
        .update_thread_seen("thread_lifecycle", "bot_reply_1", now)
        .await
        .unwrap();
    store
        .update_thread_processed("thread_lifecycle", "bot_reply_1", now)
        .await
        .unwrap();

    // 6. User reacts with ✅
    store
        .append_reaction(AppendReaction {
            thread_id: Some("thread_lifecycle".to_string()),
            post_id: "root_post_lifecycle".to_string(),
            user_id: "user_lifecycle".to_string(),
            emoji_name: "white_check_mark".to_string(),
            action: ReactionAction::Added,
        })
        .await
        .unwrap();

    // 7. Resolve thread
    store
        .update_thread_status("thread_lifecycle", ThreadStatus::Resolved)
        .await
        .unwrap();

    // Verify final state
    let final_thread = store.get_thread("thread_lifecycle").await.unwrap().unwrap();
    assert_eq!(final_thread.status, ThreadStatus::Resolved);
    assert_eq!(
        final_thread.last_seen_post_id.as_deref(),
        Some("bot_reply_1")
    );
    assert_eq!(
        final_thread.last_processed_post_id.as_deref(),
        Some("bot_reply_1")
    );

    let messages = store
        .list_thread_messages("thread_lifecycle")
        .await
        .unwrap();
    assert_eq!(messages.len(), 2);
    assert!(!messages[0].is_bot_message);
    assert!(messages[1].is_bot_message);

    let reactions = store
        .list_thread_reactions("thread_lifecycle")
        .await
        .unwrap();
    assert_eq!(reactions.len(), 1);
    assert_eq!(reactions[0].emoji_name, "white_check_mark");

    db.cleanup().await;
}

#[tokio::test]
async fn messages_cascade_delete_with_thread() {
    let db = TestDb::new().await;
    let store = db.store().await;

    // Create thread with messages and reactions
    let thread = make_thread("cascade");
    store.upsert_thread(thread).await.unwrap();

    let msg = UpsertThreadMessage {
        post_id: "cascade_msg".to_string(),
        thread_id: "thread_cascade".to_string(),
        user_id: "user_1".to_string(),
        is_bot_message: false,
        root_id: None,
        parent_post_id: None,
        metadata: json!({}),
        post_created_at: Utc::now(),
        post_updated_at: None,
        post_deleted_at: None,
    };
    store.upsert_message(msg).await.unwrap();

    store
        .append_reaction(AppendReaction {
            thread_id: Some("thread_cascade".to_string()),
            post_id: "cascade_msg".to_string(),
            user_id: "user_2".to_string(),
            emoji_name: "thumbsup".to_string(),
            action: ReactionAction::Added,
        })
        .await
        .unwrap();

    // Delete the thread directly via SQL (simulating cascade)
    sqlx::query("DELETE FROM threads WHERE thread_id = $1")
        .bind("thread_cascade")
        .execute(&db.pool)
        .await
        .unwrap();

    // Messages and reactions should be gone (CASCADE)
    let msg = store.get_message("cascade_msg").await.unwrap();
    assert!(msg.is_none());

    let reactions = store.list_thread_reactions("thread_cascade").await.unwrap();
    assert!(reactions.is_empty());

    db.cleanup().await;
}

#[tokio::test]
async fn multiple_threads_in_same_channel() {
    let db = TestDb::new().await;
    let store = db.store().await;

    // Two threads in the same channel
    let t1 = UpsertThread {
        thread_id: "thread_a".to_string(),
        root_post_id: "root_a".to_string(),
        channel_id: "shared_channel".to_string(),
        creator_user_id: "user_1".to_string(),
        status: ThreadStatus::Active,
        metadata: json!({}),
    };
    let t2 = UpsertThread {
        thread_id: "thread_b".to_string(),
        root_post_id: "root_b".to_string(),
        channel_id: "shared_channel".to_string(),
        creator_user_id: "user_2".to_string(),
        status: ThreadStatus::Active,
        metadata: json!({}),
    };
    store.upsert_thread(t1).await.unwrap();
    store.upsert_thread(t2).await.unwrap();

    // Messages in different threads
    let msg_a = UpsertThreadMessage {
        post_id: "msg_in_a".to_string(),
        thread_id: "thread_a".to_string(),
        user_id: "user_1".to_string(),
        is_bot_message: false,
        root_id: Some("root_a".to_string()),
        parent_post_id: None,
        metadata: json!({}),
        post_created_at: Utc::now(),
        post_updated_at: None,
        post_deleted_at: None,
    };
    let msg_b = UpsertThreadMessage {
        post_id: "msg_in_b".to_string(),
        thread_id: "thread_b".to_string(),
        user_id: "user_2".to_string(),
        is_bot_message: false,
        root_id: Some("root_b".to_string()),
        parent_post_id: None,
        metadata: json!({}),
        post_created_at: Utc::now(),
        post_updated_at: None,
        post_deleted_at: None,
    };
    store.upsert_message(msg_a).await.unwrap();
    store.upsert_message(msg_b).await.unwrap();

    // Each thread only has its own messages
    let msgs_a = store.list_thread_messages("thread_a").await.unwrap();
    assert_eq!(msgs_a.len(), 1);
    assert_eq!(msgs_a[0].post_id, "msg_in_a");

    let msgs_b = store.list_thread_messages("thread_b").await.unwrap();
    assert_eq!(msgs_b.len(), 1);
    assert_eq!(msgs_b[0].post_id, "msg_in_b");

    // get_thread_by_post routes to correct thread
    let found_a = store.get_thread_by_post("msg_in_a").await.unwrap().unwrap();
    assert_eq!(found_a.thread_id, "thread_a");

    let found_b = store.get_thread_by_post("msg_in_b").await.unwrap().unwrap();
    assert_eq!(found_b.thread_id, "thread_b");

    db.cleanup().await;
}

#[tokio::test]
async fn metadata_supports_complex_json() {
    let db = TestDb::new().await;
    let store = db.store().await;

    let thread = UpsertThread {
        thread_id: "thread_json".to_string(),
        root_post_id: "root_json".to_string(),
        channel_id: "ch_1".to_string(),
        creator_user_id: "user_1".to_string(),
        status: ThreadStatus::New,
        metadata: json!({
            "tags": ["urgent", "bug"],
            "assignee": {"name": "Alice", "id": 42},
            "nested": {"a": {"b": {"c": true}}}
        }),
    };
    let record = store.upsert_thread(thread).await.unwrap();

    assert_eq!(record.metadata["tags"][0], "urgent");
    assert_eq!(record.metadata["assignee"]["name"], "Alice");
    assert_eq!(record.metadata["nested"]["a"]["b"]["c"], true);

    db.cleanup().await;
}
