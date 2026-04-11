//! Tests for the per-thread actor: helper functions and actor behavior.

use std::time::Duration;

use chrono::Utc;
use mattermost_api::models;

use crate::actor::{
    get_root_post_id, is_root_post, ms_to_datetime, post_to_thread_message, ThreadCommand,
};
use crate::handler::{ThreadCloseReason, ThreadEffect};
use crate::testutil::{make_mm_post, spawn_test_actor, MockHandler};
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

// ── Actor behavior tests ─────────────────────────────────────────────────────

/// Short debounce for tests.
const TEST_DEBOUNCE: Duration = Duration::from_millis(50);

/// Wait long enough for debounce + handler to complete.
async fn settle() {
    tokio::time::sleep(Duration::from_millis(300)).await;
}

#[tokio::test]
async fn actor_basic_handler_flow() {
    let handler = MockHandler::new().with_default_effects(vec![ThreadEffect::Reply {
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
    assert!(matches!(thread.status, ThreadStatus::Active));
    assert!(thread.last_processed_post_id.is_some());

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
async fn actor_control_reaction_resolved() {
    let handler = MockHandler::new();
    let post = make_mm_post("thread1", "user_1", "root", None);
    let (tx, store, handler, _server) =
        spawn_test_actor(handler, "thread1", vec![post], TEST_DEBOUNCE).await;

    tx.send(ThreadCommand::ControlReaction {
        effect: ThreadEffect::MarkResolved,
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
    assert!(matches!(thread.status, ThreadStatus::Resolved));

    let reasons = handler.closed_reasons.lock().await;
    assert_eq!(reasons.len(), 1);
    assert_eq!(reasons[0], ThreadCloseReason::ResolvedByReaction);

    assert!(tx.is_closed());
}

#[tokio::test]
async fn actor_control_reaction_stopped() {
    let handler = MockHandler::new();
    let post = make_mm_post("thread1", "user_1", "root", None);
    let (tx, store, handler, _server) =
        spawn_test_actor(handler, "thread1", vec![post], TEST_DEBOUNCE).await;

    tx.send(ThreadCommand::ControlReaction {
        effect: ThreadEffect::MarkStopped,
        change: ReactionChange {
            post_id: "thread1".into(),
            user_id: "user_2".into(),
            emoji_name: "stop_sign".into(),
            action: ReactionAction::Added,
            created_at: Utc::now(),
        },
    })
    .await
    .unwrap();

    settle().await;

    let thread = store.thread_snapshot("thread1").await.unwrap();
    assert!(matches!(thread.status, ThreadStatus::Stopped));

    let reasons = handler.closed_reasons.lock().await;
    assert_eq!(reasons.len(), 1);
    assert_eq!(reasons[0], ThreadCloseReason::StoppedByReaction);

    assert!(tx.is_closed());
}

#[tokio::test]
async fn actor_handler_returns_mark_resolved() {
    let handler = MockHandler::new().with_default_effects(vec![ThreadEffect::MarkResolved]);

    let post = make_mm_post("p1", "user_1", "hello", Some("thread1"));
    let (tx, store, handler, _server) =
        spawn_test_actor(handler, "thread1", vec![post.clone()], TEST_DEBOUNCE).await;

    tx.send(ThreadCommand::NewMessage { post }).await.unwrap();
    settle().await;

    let thread = store.thread_snapshot("thread1").await.unwrap();
    assert!(matches!(thread.status, ThreadStatus::Resolved));

    let reasons = handler.closed_reasons.lock().await;
    assert_eq!(reasons[0], ThreadCloseReason::ResolvedByHandler);

    assert!(tx.is_closed());
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
async fn actor_new_to_active_transition() {
    let handler = MockHandler::new();
    let post = make_mm_post("p1", "user_1", "hello", Some("thread1"));
    let (tx, store, _handler, _server) =
        spawn_test_actor(handler, "thread1", vec![post.clone()], TEST_DEBOUNCE).await;

    let thread = store.thread_snapshot("thread1").await.unwrap();
    assert!(matches!(thread.status, ThreadStatus::New));

    tx.send(ThreadCommand::NewMessage { post }).await.unwrap();
    settle().await;

    let thread = store.thread_snapshot("thread1").await.unwrap();
    assert!(matches!(thread.status, ThreadStatus::Active));

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
        effect: ThreadEffect::MarkResolved,
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
    assert!(matches!(thread.status, ThreadStatus::Resolved));

    assert!(tx.is_closed());
}

#[tokio::test]
async fn actor_multiple_effects_in_one_return() {
    let handler = MockHandler::new().with_default_effects(vec![
        ThreadEffect::Reply {
            message: "thinking...".into(),
            metadata: serde_json::Value::Null,
        },
        ThreadEffect::SetThreadMetadata {
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
