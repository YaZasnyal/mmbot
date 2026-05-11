//! Tests for the per-thread actor: helper functions and actor behavior.

use std::time::Duration;

use chrono::Utc;
use mattermost_api::models;

use crate::actor::{
    build_thread_snapshot, get_root_post_id, is_root_post, ms_to_datetime, post_to_thread_message,
    post_to_upsert_thread_message, ThreadCommand,
};
use crate::handler::{ThreadEffect, ThreadMetadataTarget, ThreadTarget};
use crate::store::ThreadStore;
use crate::testutil::{
    make_mm_post, setup_mm_mock, spawn_test_actor, spawn_test_actor_with_idle_timeout, MockHandler,
    MockStore,
};
use crate::types::*;

// ── Helper function unit tests ───────────────────────────────────────────────

#[test]
fn ms_to_datetime_roundtrip() {
    let ts = 1_700_000_000_000i64;
    let dt = ms_to_datetime(ts);
    assert_eq!(dt.timestamp_millis(), ts);
}

#[test]
fn ms_to_datetime_zero() {
    let dt = ms_to_datetime(0);
    assert_eq!(dt.timestamp(), 0);
}

#[test]
fn is_root_post_none_root_id() {
    let post = models::Post {
        id: "p1".into(),
        root_id: None,
        ..Default::default()
    };
    assert!(is_root_post(&post));
}

#[test]
fn is_root_post_empty_root_id() {
    let post = models::Post {
        id: "p1".into(),
        root_id: Some(String::new()),
        ..Default::default()
    };
    assert!(is_root_post(&post));
}

#[test]
fn is_root_post_with_root_id() {
    let post = models::Post {
        id: "p1".into(),
        root_id: Some("root".into()),
        ..Default::default()
    };
    assert!(!is_root_post(&post));
}

#[test]
fn get_root_post_id_for_root() {
    let post = models::Post {
        id: "p1".into(),
        root_id: None,
        ..Default::default()
    };
    assert_eq!(get_root_post_id(&post), "p1");
}

#[test]
fn get_root_post_id_for_reply() {
    let post = models::Post {
        id: "p1".into(),
        root_id: Some("root".into()),
        ..Default::default()
    };
    assert_eq!(get_root_post_id(&post), "root");
}

#[test]
fn post_to_thread_message_preserves_fields() {
    let post = models::Post {
        id: "p1".into(),
        user_id: Some("u1".into()),
        message: Some("hello".into()),
        root_id: Some("root".into()),
        create_at: Some(1_700_000_000_000),
        update_at: Some(1_700_000_001_000),
        props: Some(serde_json::json!({"key": "val"})),
        ..Default::default()
    };

    let msg = post_to_thread_message(&post, "t1");
    assert_eq!(msg.post_id, "p1");
    assert_eq!(msg.thread_id, "t1");
    assert_eq!(msg.user_id, "u1");
    assert_eq!(msg.message, "hello");
    assert_eq!(msg.root_id, Some("root".into()));
    assert_eq!(msg.props, serde_json::json!({"key": "val"}));
    assert!(!msg.is_new);
}

#[test]
fn post_to_thread_message_defaults_for_missing() {
    let post = models::Post {
        id: "p1".into(),
        ..Default::default()
    };

    let msg = post_to_thread_message(&post, "t1");
    assert_eq!(msg.user_id, "");
    assert_eq!(msg.message, "");
    assert_eq!(msg.props, serde_json::Value::Null);
}

#[test]
fn post_to_upsert_thread_message_preserves_persistence_fields() {
    let post = models::Post {
        id: "p1".into(),
        user_id: Some("bot".into()),
        root_id: Some("root".into()),
        create_at: Some(1_700_000_000_000),
        update_at: Some(1_700_000_001_000),
        delete_at: Some(0),
        props: Some(serde_json::json!({"key": "val"})),
        ..Default::default()
    };

    let msg = post_to_upsert_thread_message(&post, "t1", true);
    assert_eq!(msg.post_id, "p1");
    assert_eq!(msg.thread_id, "t1");
    assert_eq!(msg.user_id, "bot");
    assert!(msg.is_bot_message);
    assert_eq!(msg.root_id, Some("root".into()));
    assert_eq!(msg.parent_post_id, Some("root".into()));
    assert_eq!(msg.metadata, serde_json::json!({"key": "val"}));
    assert_eq!(msg.post_created_at, ms_to_datetime(1_700_000_000_000));
    assert_eq!(msg.post_updated_at, Some(ms_to_datetime(1_700_000_001_000)));
    assert_eq!(msg.post_deleted_at, None);
}

#[tokio::test]
async fn build_thread_snapshot_orders_messages_and_merges_metadata() {
    let mut second = make_mm_post("post-2", "user_1", "second", Some("thread1"));
    second.create_at = Some(2_000);
    let mut first = make_mm_post("post-1", "user_1", "first", Some("thread1"));
    first.create_at = Some(1_000);
    let (_server, mm_config) = setup_mm_mock(vec![second, first]).await;
    let store = MockStore::new();

    store
        .upsert_thread(UpsertThread {
            thread_id: "thread1".to_string(),
            root_post_id: "thread1".to_string(),
            channel_id: "test_channel".to_string(),
            creator_user_id: "user_1".to_string(),
            thread_kind: None,
            metadata: serde_json::Value::Null,
        })
        .await
        .unwrap();
    store
        .upsert_message(UpsertThreadMessage {
            post_id: "post-1".to_string(),
            thread_id: "thread1".to_string(),
            user_id: "user_1".to_string(),
            is_bot_message: false,
            root_id: Some("thread1".to_string()),
            parent_post_id: Some("thread1".to_string()),
            metadata: serde_json::json!({ "trace": "kept" }),
            post_created_at: ms_to_datetime(1_000),
            post_updated_at: None,
            post_deleted_at: None,
        })
        .await
        .unwrap();

    let thread = build_thread_snapshot("thread1", &store, &mm_config)
        .await
        .unwrap();

    assert_eq!(thread.messages[0].post_id, "post-1");
    assert_eq!(thread.messages[1].post_id, "post-2");
    assert_eq!(
        thread.messages[0].metadata,
        serde_json::json!({ "trace": "kept" })
    );
}

// ── Actor behavior tests ─────────────────────────────────────────────────────

/// Short debounce for tests.
const TEST_DEBOUNCE: Duration = Duration::from_millis(50);

/// Wait long enough for debounce + handler to complete.
async fn settle() {
    tokio::time::sleep(Duration::from_millis(300)).await;
}

async fn wait_actor_closed(tx: &tokio::sync::mpsc::Sender<ThreadCommand>) {
    tokio::time::timeout(Duration::from_secs(1), async {
        while !tx.is_closed() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("actor did not close in time");
}

#[tokio::test]
async fn actor_basic_handler_flow() {
    let handler = MockHandler::new().with_default_effects(vec![ThreadEffect::Reply {
        target: ThreadTarget::CurrentThread,
        message: "response".into(),
        metadata: serde_json::Value::Null,
    }]);

    let post = make_mm_post("post1", "user_1", "hello", Some("thread1"));
    let (tx, store, handler, _server) =
        spawn_test_actor(handler, "thread1", vec![post.clone()], TEST_DEBOUNCE).await;

    tx.send(ThreadCommand::NewMessage { post }).await.unwrap();
    settle().await;

    assert_eq!(handler.call_count(), 1);

    let thread = store.thread_snapshot("thread1").await.unwrap();
    assert!(thread.last_processed_post_id.is_some());
    let reply = store
        .message_snapshot("reply_post_id")
        .await
        .expect("bot reply should be persisted immediately");
    assert_eq!(reply.thread_id, "thread1");
    assert!(reply.is_bot_message);
    assert_eq!(reply.metadata, serde_json::Value::Null);

    drop(tx);
}

#[tokio::test]
async fn actor_debounce_coalesces_messages() {
    let handler = MockHandler::new();
    let post1 = make_mm_post("p1", "user_1", "msg1", Some("thread1"));
    let post2 = make_mm_post("p2", "user_1", "msg2", Some("thread1"));
    let post3 = make_mm_post("p3", "user_1", "msg3", Some("thread1"));

    let (tx, _store, handler, _server) = spawn_test_actor(
        handler,
        "thread1",
        vec![post1.clone(), post2.clone(), post3.clone()],
        Duration::from_millis(100),
    )
    .await;

    // Three messages in quick succession (within debounce window)
    tx.send(ThreadCommand::NewMessage { post: post1 })
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(10)).await;
    tx.send(ThreadCommand::NewMessage { post: post2 })
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(10)).await;
    tx.send(ThreadCommand::NewMessage { post: post3 })
        .await
        .unwrap();

    settle().await;

    // Handler called exactly once
    assert_eq!(handler.call_count(), 1);

    drop(tx);
}

#[tokio::test]
async fn actor_handler_interrupted_by_new_message() {
    // Handler takes 5s — we'll interrupt it well before that
    let handler = MockHandler::new().with_handle_delay(Duration::from_secs(5));

    let post1 = make_mm_post("p1", "user_1", "msg1", Some("thread1"));
    let post2 = make_mm_post("p2", "user_1", "msg2", Some("thread1"));

    let (tx, _store, handler, _server) = spawn_test_actor(
        handler,
        "thread1",
        vec![post1.clone(), post2.clone()],
        TEST_DEBOUNCE,
    )
    .await;

    tx.send(ThreadCommand::NewMessage { post: post1 })
        .await
        .unwrap();

    // Wait for handler to enter
    handler.wait_handle_entered().await;

    // Send second message — interrupts the running handler
    tx.send(ThreadCommand::NewMessage { post: post2 })
        .await
        .unwrap();

    // Wait for second invocation to enter
    handler.wait_handle_entered().await;

    assert_eq!(handler.call_count(), 2);

    drop(tx);
}

#[tokio::test]
async fn actor_shutdown_exits() {
    let handler = MockHandler::new();
    let (tx, _store, _handler, _server) =
        spawn_test_actor(handler, "thread1", vec![], TEST_DEBOUNCE).await;

    tx.send(ThreadCommand::Shutdown).await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert!(tx.is_closed());
}

#[tokio::test]
async fn actor_channel_closed_exits() {
    let handler = MockHandler::new();
    let (tx, _store, _handler, _server) =
        spawn_test_actor(handler, "thread1", vec![], TEST_DEBOUNCE).await;

    drop(tx);
    // Actor exits gracefully — no panic, no hang
    tokio::time::sleep(Duration::from_millis(50)).await;
}

#[tokio::test]
async fn actor_idle_timeout_zero_exits_after_handler() {
    let handler = MockHandler::new();
    let post = make_mm_post("p1", "user_1", "hello", Some("thread1"));

    let (tx, _store, handler, _server) = spawn_test_actor_with_idle_timeout(
        handler,
        "thread1",
        vec![post.clone()],
        TEST_DEBOUNCE,
        Duration::ZERO,
    )
    .await;

    tx.send(ThreadCommand::NewMessage { post }).await.unwrap();
    handler.wait_handle_entered().await;
    wait_actor_closed(&tx).await;

    assert_eq!(handler.call_count(), 1);
}

#[tokio::test]
async fn actor_idle_timeout_waits_for_configured_delay() {
    let handler = MockHandler::new();
    let post = make_mm_post("p1", "user_1", "hello", Some("thread1"));

    let (tx, _store, handler, _server) = spawn_test_actor_with_idle_timeout(
        handler,
        "thread1",
        vec![post.clone()],
        TEST_DEBOUNCE,
        Duration::from_millis(200),
    )
    .await;

    tx.send(ThreadCommand::NewMessage { post }).await.unwrap();
    handler.wait_handle_entered().await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert!(!tx.is_closed());

    wait_actor_closed(&tx).await;
}

#[tokio::test]
async fn actor_idle_timeout_does_not_fire_while_pending() {
    let handler = MockHandler::new();
    let post = make_mm_post("p1", "user_1", "hello", Some("thread1"));

    let (tx, _store, handler, _server) = spawn_test_actor_with_idle_timeout(
        handler,
        "thread1",
        vec![post.clone()],
        Duration::from_millis(200),
        Duration::from_millis(10),
    )
    .await;

    tx.send(ThreadCommand::NewMessage { post }).await.unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert!(!tx.is_closed());
    assert_eq!(handler.call_count(), 0);

    handler.wait_handle_entered().await;
    wait_actor_closed(&tx).await;
}

#[tokio::test]
async fn actor_idle_timeout_does_not_fire_while_handler_runs() {
    let handler = MockHandler::new().with_handle_delay(Duration::from_millis(200));
    let post = make_mm_post("p1", "user_1", "hello", Some("thread1"));

    let (tx, _store, handler, _server) = spawn_test_actor_with_idle_timeout(
        handler,
        "thread1",
        vec![post.clone()],
        Duration::from_millis(10),
        Duration::from_millis(10),
    )
    .await;

    tx.send(ThreadCommand::NewMessage { post }).await.unwrap();
    handler.wait_handle_entered().await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert!(!tx.is_closed());

    wait_actor_closed(&tx).await;
}

#[tokio::test]
async fn actor_reschedule_runs_handler_again() {
    let handler = MockHandler::new().with_effects_queue(vec![
        vec![ThreadEffect::Noop, ThreadEffect::Reschedule],
        vec![ThreadEffect::Noop],
    ]);

    let post = make_mm_post("p1", "user_1", "hello", Some("thread1"));
    let (tx, _store, handler, _server) =
        spawn_test_actor(handler, "thread1", vec![post.clone()], TEST_DEBOUNCE).await;

    tx.send(ThreadCommand::NewMessage { post }).await.unwrap();

    handler.wait_handle_entered().await; // first run
    handler.wait_handle_entered().await; // reschedule

    assert_eq!(handler.call_count(), 2);

    drop(tx);
}

#[tokio::test]
async fn actor_handler_error_continues() {
    let handler = MockHandler::new().with_error();

    let post1 = make_mm_post("p1", "user_1", "msg1", Some("thread1"));
    let post2 = make_mm_post("p2", "user_1", "msg2", Some("thread1"));

    let (tx, _store, handler, _server) = spawn_test_actor(
        handler,
        "thread1",
        vec![post1.clone(), post2.clone()],
        TEST_DEBOUNCE,
    )
    .await;

    tx.send(ThreadCommand::NewMessage { post: post1 })
        .await
        .unwrap();
    settle().await;
    assert_eq!(handler.call_count(), 1);

    // Actor still alive
    tx.send(ThreadCommand::NewMessage { post: post2 })
        .await
        .unwrap();
    settle().await;
    assert_eq!(handler.call_count(), 2);

    assert!(!tx.is_closed());

    drop(tx);
}

#[tokio::test]
async fn actor_bot_message_does_not_trigger_handler() {
    let handler = MockHandler::new();
    // bot_user_id is "bot_user" in spawn_test_actor
    let post = make_mm_post("p1", "bot_user", "I'm the bot", Some("thread1"));

    let (tx, store, handler, _server) =
        spawn_test_actor(handler, "thread1", vec![post.clone()], TEST_DEBOUNCE).await;

    tx.send(ThreadCommand::NewMessage { post }).await.unwrap();
    settle().await;

    assert_eq!(handler.call_count(), 0);

    let msg = store.message_snapshot("p1").await;
    assert!(msg.is_some());
    assert!(msg.unwrap().is_bot_message);

    drop(tx);
}

#[tokio::test]
async fn actor_set_thread_metadata_effect() {
    let handler = MockHandler::new().with_default_effects(vec![ThreadEffect::SetThreadMetadata {
        target: ThreadMetadataTarget::CurrentThread,
        metadata: serde_json::json!({"llm_model": "gpt-4"}),
    }]);

    let post = make_mm_post("p1", "user_1", "hello", Some("thread1"));
    let (tx, store, _handler, _server) =
        spawn_test_actor(handler, "thread1", vec![post.clone()], TEST_DEBOUNCE).await;

    tx.send(ThreadCommand::NewMessage { post }).await.unwrap();
    settle().await;

    let thread = store.thread_snapshot("thread1").await.unwrap();
    assert_eq!(thread.metadata, serde_json::json!({"llm_model": "gpt-4"}));

    drop(tx);
}

#[tokio::test]
async fn actor_set_thread_metadata_effect_supports_explicit_targets() {
    let handler = MockHandler::new().with_default_effects(vec![
        ThreadEffect::SetThreadMetadata {
            target: ThreadMetadataTarget::ThreadId("target-thread".into()),
            metadata: serde_json::json!({"target": "thread_id"}),
        },
        ThreadEffect::SetThreadMetadata {
            target: ThreadMetadataTarget::RootPostId("target-root".into()),
            metadata: serde_json::json!({"target": "root_post_id"}),
        },
    ]);

    let post = make_mm_post("p1", "user_1", "hello", Some("thread1"));
    let (tx, store, _handler, _server) =
        spawn_test_actor(handler, "thread1", vec![post.clone()], TEST_DEBOUNCE).await;
    store
        .upsert_thread(UpsertThread {
            thread_id: "target-thread".into(),
            root_post_id: "target-thread".into(),
            channel_id: "target-channel".into(),
            creator_user_id: "bot_user".into(),
            thread_kind: Some("target".into()),
            metadata: serde_json::json!({}),
        })
        .await
        .unwrap();
    store
        .upsert_thread(UpsertThread {
            thread_id: "root-target-thread".into(),
            root_post_id: "target-root".into(),
            channel_id: "target-channel".into(),
            creator_user_id: "bot_user".into(),
            thread_kind: Some("target".into()),
            metadata: serde_json::json!({}),
        })
        .await
        .unwrap();

    tx.send(ThreadCommand::NewMessage { post }).await.unwrap();
    settle().await;

    let target_by_id = store.thread_snapshot("target-thread").await.unwrap();
    assert_eq!(
        target_by_id.metadata,
        serde_json::json!({"target": "thread_id"})
    );
    let target_by_root = store.thread_snapshot("root-target-thread").await.unwrap();
    assert_eq!(
        target_by_root.metadata,
        serde_json::json!({"target": "root_post_id"})
    );

    drop(tx);
}

#[tokio::test]
async fn actor_keeps_running_when_critical_metadata_effect_fails() {
    let handler = MockHandler::new().with_default_effects(vec![
        ThreadEffect::SetMessageMetadata {
            post_id: "missing-post".to_string(),
            metadata: serde_json::json!({"trace": []}),
        },
        ThreadEffect::SetThreadMetadata {
            target: ThreadMetadataTarget::CurrentThread,
            metadata: serde_json::json!({"should_not": "run"}),
        },
    ]);

    let post = make_mm_post("p1", "user_1", "hello", Some("thread1"));
    let (tx, store, _handler, _server) =
        spawn_test_actor(handler, "thread1", vec![post.clone()], TEST_DEBOUNCE).await;

    tx.send(ThreadCommand::NewMessage { post }).await.unwrap();
    settle().await;

    let thread = store.thread_snapshot("thread1").await.unwrap();
    assert_eq!(thread.metadata, serde_json::Value::Null);

    assert!(!tx.is_closed());
}

#[tokio::test]
async fn actor_control_reaction_interrupts_handler() {
    let handler = MockHandler::new().with_handle_delay(Duration::from_secs(5));

    let post = make_mm_post("thread1", "user_1", "root", None);
    let (tx, store, _handler, _server) =
        spawn_test_actor(handler, "thread1", vec![post.clone()], TEST_DEBOUNCE).await;

    tx.send(ThreadCommand::NewMessage {
        post: make_mm_post("p1", "user_1", "msg", Some("thread1")),
    })
    .await
    .unwrap();

    // Give enough time for debounce to fire and handler to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    tx.send(ThreadCommand::ControlReaction {
        effect: ThreadEffect::SetThreadMetadata {
            target: ThreadMetadataTarget::CurrentThread,
            metadata: serde_json::json!({
                "support_bot": {
                    "status": "finished"
                }
            }),
        },
        change: ReactionChange {
            post_id: "thread1".into(),
            user_id: "user_2".into(),
            emoji_name: "white_check_mark".into(),
            action: ReactionAction::Added,
            created_at: Utc::now(),
        },
    })
    .await
    .unwrap();

    settle().await;

    let thread = store.thread_snapshot("thread1").await.unwrap();
    assert_eq!(thread.metadata["support_bot"]["status"], "finished");

    assert!(!tx.is_closed());
}

#[tokio::test]
async fn actor_control_reaction_executes_metadata_effect() {
    let handler = MockHandler::new();

    let post = make_mm_post("thread1", "user_1", "root", None);
    let (tx, store, _handler, _server) =
        spawn_test_actor(handler, "thread1", vec![post], TEST_DEBOUNCE).await;

    tx.send(ThreadCommand::ControlReaction {
        effect: ThreadEffect::SetThreadMetadata {
            target: ThreadMetadataTarget::CurrentThread,
            metadata: serde_json::json!({
                "support_bot": {
                    "status": "finished"
                }
            }),
        },
        change: ReactionChange {
            post_id: "thread1".into(),
            user_id: "user_2".into(),
            emoji_name: "white_check_mark".into(),
            action: ReactionAction::Added,
            created_at: Utc::now(),
        },
    })
    .await
    .unwrap();

    settle().await;

    let thread = store.thread_snapshot("thread1").await.unwrap();
    assert_eq!(thread.metadata["support_bot"]["status"], "finished");
    assert!(!tx.is_closed());
}

#[tokio::test]
async fn actor_multiple_effects_in_one_return() {
    let handler = MockHandler::new().with_default_effects(vec![
        ThreadEffect::Reply {
            target: ThreadTarget::CurrentThread,
            message: "thinking...".into(),
            metadata: serde_json::Value::Null,
        },
        ThreadEffect::SetThreadMetadata {
            target: ThreadMetadataTarget::CurrentThread,
            metadata: serde_json::json!({"step": 1}),
        },
    ]);

    let post = make_mm_post("p1", "user_1", "hello", Some("thread1"));
    let (tx, store, handler, _server) =
        spawn_test_actor(handler, "thread1", vec![post.clone()], TEST_DEBOUNCE).await;

    tx.send(ThreadCommand::NewMessage { post }).await.unwrap();
    settle().await;

    assert_eq!(handler.call_count(), 1);

    let thread = store.thread_snapshot("thread1").await.unwrap();
    assert_eq!(thread.metadata, serde_json::json!({"step": 1}));

    drop(tx);
}

#[tokio::test]
async fn actor_ensure_linked_thread_creates_tracked_thread_and_link() {
    let handler = MockHandler::new().with_default_effects(vec![ThreadEffect::EnsureLinkedThread {
        link_kind: "engineer".into(),
        channel_id: "engineer_channel".into(),
        thread_kind: Some("support_engineer".into()),
        message: "Engineer thread".into(),
        metadata: serde_json::json!({"kind": "engineer_root"}),
        thread_metadata: serde_json::json!({"status": "open"}),
    }]);

    let post = make_mm_post("p1", "user_1", "hello", Some("thread1"));
    let (tx, store, handler, _server) =
        spawn_test_actor(handler, "thread1", vec![post.clone()], TEST_DEBOUNCE).await;

    tx.send(ThreadCommand::NewMessage { post }).await.unwrap();
    settle().await;

    assert_eq!(handler.call_count(), 1);

    let link = store
        .list_thread_links("thread1")
        .await
        .unwrap()
        .into_iter()
        .find(|link| link.link_kind == "engineer")
        .expect("link should be stored");
    assert_eq!(link.target_thread_id, "reply_post_id");

    let linked_thread = store
        .thread_snapshot("reply_post_id")
        .await
        .expect("linked thread should be tracked");
    assert_eq!(linked_thread.channel_id, "engineer_channel");
    assert_eq!(
        linked_thread.thread_kind.as_deref(),
        Some("support_engineer")
    );
    assert_eq!(
        linked_thread.metadata,
        serde_json::json!({"status": "open"})
    );

    let root_message = store
        .message_snapshot("reply_post_id")
        .await
        .expect("linked root message should be stored");
    assert_eq!(root_message.thread_id, "reply_post_id");
    assert!(root_message.is_bot_message);
    assert_eq!(
        root_message.metadata,
        serde_json::json!({"kind": "engineer_root"})
    );

    drop(tx);
}

#[tokio::test]
async fn actor_reply_to_linked_threads_uses_stored_links() {
    let handler = MockHandler::new().with_default_effects(vec![ThreadEffect::Reply {
        target: ThreadTarget::LinkedThreads {
            link_kind: "engineer".into(),
        },
        message: "linked reply".into(),
        metadata: serde_json::json!({"kind": "linked_reply"}),
    }]);

    let post = make_mm_post("p1", "user_1", "hello", Some("thread1"));
    let (tx, store, handler, _server) =
        spawn_test_actor(handler, "thread1", vec![post.clone()], TEST_DEBOUNCE).await;
    store
        .upsert_thread(UpsertThread {
            thread_id: "engineer-thread".into(),
            root_post_id: "engineer-root".into(),
            channel_id: "engineer-channel".into(),
            creator_user_id: "bot_user".into(),
            thread_kind: Some("support_engineer".into()),
            metadata: serde_json::json!({}),
        })
        .await
        .unwrap();
    store
        .upsert_thread_link(UpsertThreadLink {
            source_thread_id: "thread1".into(),
            link_kind: "engineer".into(),
            target_thread_id: "engineer-thread".into(),
        })
        .await
        .unwrap();

    tx.send(ThreadCommand::NewMessage { post }).await.unwrap();
    settle().await;

    assert_eq!(handler.call_count(), 1);
    let reply = store
        .message_snapshot("reply_post_id")
        .await
        .expect("linked reply should be persisted");
    assert_eq!(reply.thread_id, "engineer-thread");
    assert_eq!(reply.root_id.as_deref(), Some("engineer-root"));
    assert_eq!(reply.metadata, serde_json::json!({"kind": "linked_reply"}));

    drop(tx);
}

#[tokio::test]
async fn actor_reply_to_thread_uses_explicit_thread_id() {
    let handler = MockHandler::new().with_default_effects(vec![ThreadEffect::Reply {
        target: ThreadTarget::Thread {
            thread_id: "engineer-thread".into(),
        },
        message: "direct reply".into(),
        metadata: serde_json::json!({"kind": "direct_reply"}),
    }]);

    let post = make_mm_post("p1", "user_1", "hello", Some("thread1"));
    let (tx, store, handler, _server) =
        spawn_test_actor(handler, "thread1", vec![post.clone()], TEST_DEBOUNCE).await;
    store
        .upsert_thread(UpsertThread {
            thread_id: "engineer-thread".into(),
            root_post_id: "engineer-root".into(),
            channel_id: "engineer-channel".into(),
            creator_user_id: "bot_user".into(),
            thread_kind: Some("support_engineer".into()),
            metadata: serde_json::json!({}),
        })
        .await
        .unwrap();

    tx.send(ThreadCommand::NewMessage { post }).await.unwrap();
    settle().await;

    assert_eq!(handler.call_count(), 1);
    let reply = store
        .message_snapshot("reply_post_id")
        .await
        .expect("direct reply should be persisted");
    assert_eq!(reply.thread_id, "engineer-thread");
    assert_eq!(reply.root_id.as_deref(), Some("engineer-root"));
    assert_eq!(reply.metadata, serde_json::json!({"kind": "direct_reply"}));

    drop(tx);
}

#[tokio::test]
async fn actor_seen_position_updated_on_message() {
    let handler = MockHandler::new();
    let post = make_mm_post("p1", "user_1", "hello", Some("thread1"));

    let (tx, store, _handler, _server) =
        spawn_test_actor(handler, "thread1", vec![post.clone()], TEST_DEBOUNCE).await;

    tx.send(ThreadCommand::NewMessage { post }).await.unwrap();

    // seen is updated immediately (before debounce fires)
    tokio::time::sleep(Duration::from_millis(50)).await;

    let thread = store.thread_snapshot("thread1").await.unwrap();
    assert_eq!(thread.last_seen_post_id.as_deref(), Some("p1"));

    drop(tx);
}
