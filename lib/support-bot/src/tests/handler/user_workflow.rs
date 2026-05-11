use super::*;

#[tokio::test]
async fn user_message_gets_llm_reply_and_state_update() {
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        Arc::new(StaticLlm),
        Arc::new(ToolRegistry::new()),
        "system",
    );

    let effects = handle_thread(&handler, thread("users", "help"))
        .await
        .unwrap();

    assert!(effects.iter().any(|effect| {
        matches!(effect, ThreadEffect::Reply { message, .. } if message == "hello from support")
    }));
    assert!(effects
        .iter()
        .any(|effect| matches!(effect, ThreadEffect::SetThreadMetadata { .. })));
}

#[tokio::test]
async fn empty_user_message_does_not_call_llm() {
    let llm = Arc::new(SequenceLlm::new(vec![LlmResponse {
        message: ChatMessage::assistant("should not be used"),
        tool_calls: Vec::new(),
    }]));
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        llm.clone(),
        Arc::new(ToolRegistry::new()),
        "system",
    );

    let effects = handle_thread(&handler, thread("users", "")).await.unwrap();

    assert!(matches!(effects.as_slice(), [ThreadEffect::Noop]));
    assert!(llm.requests().is_empty());
}

#[tokio::test]
async fn assistant_thinking_is_not_sent_to_user() {
    let llm = Arc::new(SequenceLlm::new(vec![LlmResponse {
        message: ChatMessage::assistant(
            "<think>private reasoning</think>\nPlease share the request id.",
        ),
        tool_calls: Vec::new(),
    }]));
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        llm,
        Arc::new(ToolRegistry::new()),
        "system",
    );

    let effects = handle_thread(&handler, thread("users", "help"))
        .await
        .unwrap();

    assert!(effects.iter().any(|effect| {
        matches!(
            effect,
            ThreadEffect::Reply { message, .. }
                if message == "Please share the request id."
        )
    }));
    assert!(!effects.iter().any(|effect| {
        matches!(
            effect,
            ThreadEffect::Reply { message, .. }
                if message.contains("private reasoning") || message.contains("<think>")
        )
    }));
}

#[tokio::test]
async fn user_tool_trace_is_not_written_via_update_message() {
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "send_user_message".to_string(),
        arguments: json!({ "message": "please send request id" }),
    };
    let llm = Arc::new(SequenceLlm::new(vec![
        LlmResponse {
            message: ChatMessage {
                role: ChatRole::Assistant,
                content: None,
                name: None,
                tool_call_id: None,
                tool_calls: vec![call.clone()],
            },
            tool_calls: vec![call],
        },
        LlmResponse {
            message: ChatMessage::assistant("done"),
            tool_calls: Vec::new(),
        },
    ]));
    let mut registry = ToolRegistry::new();
    register_default_workflow_tools(&mut registry).unwrap();
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        llm.clone(),
        Arc::new(registry),
        "system",
    );

    let effects = handle_thread(&handler, thread("users", "help"))
        .await
        .unwrap();
    assert!(!effects
        .iter()
        .any(|effect| matches!(effect, ThreadEffect::UpdateMessage { .. })));
    assert_eq!(
            effects
                .iter()
                .filter(|effect| {
                    matches!(effect, ThreadEffect::Reply { message, .. } if message == "please send request id")
                })
                .count(),
            1
        );
    assert_eq!(llm.requests().len(), 1);
}

#[tokio::test]
async fn tool_call_overflow_is_visible_to_next_llm_round() {
    let calls = vec![
        ToolCall {
            id: "call-1".to_string(),
            name: "echo_read".to_string(),
            arguments: json!({ "message": "first" }),
        },
        ToolCall {
            id: "call-2".to_string(),
            name: "echo_read".to_string(),
            arguments: json!({ "message": "second" }),
        },
    ];
    let llm = Arc::new(SequenceLlm::new(vec![
        LlmResponse {
            message: ChatMessage {
                role: ChatRole::Assistant,
                content: None,
                name: None,
                tool_call_id: None,
                tool_calls: calls.clone(),
            },
            tool_calls: calls,
        },
        LlmResponse {
            message: ChatMessage::assistant("done"),
            tool_calls: Vec::new(),
        },
    ]));
    let mut registry = ToolRegistry::new();
    registry.register(EchoReadOnlyTool).unwrap();
    let mut config = test_config();
    config.limits.max_tool_calls_per_round = 1;
    let handler =
        SupportBotHandler::new("support", config, llm.clone(), Arc::new(registry), "system");

    handle_thread(&handler, thread("users", "help"))
        .await
        .unwrap();

    let requests = llm.requests();
    assert_eq!(requests.len(), 2);
    let second_round_messages = &requests[1].messages;
    let assistant = second_round_messages
        .iter()
        .find(|message| message.role == ChatRole::Assistant && !message.tool_calls.is_empty())
        .expect("assistant tool-call message should be preserved");
    assert_eq!(assistant.tool_calls.len(), 2);

    let overflow_result = second_round_messages
        .iter()
        .find(|message| message.tool_call_id.as_deref() == Some("call-2"))
        .and_then(|message| message.content.as_deref())
        .expect("overflow tool result should be sent back to the model");
    assert!(overflow_result.contains("max_tool_calls_per_round"));
    assert!(overflow_result.contains("\"is_error\":true"));
}

#[tokio::test]
async fn send_user_message_tool_strips_hidden_reasoning() {
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "send_user_message".to_string(),
        arguments: json!({
            "message": "<thinking>private reasoning</thinking>\nPlease send request id"
        }),
    };
    let llm = Arc::new(SequenceLlm::new(vec![
        LlmResponse {
            message: ChatMessage {
                role: ChatRole::Assistant,
                content: None,
                name: None,
                tool_call_id: None,
                tool_calls: vec![call.clone()],
            },
            tool_calls: vec![call],
        },
        LlmResponse {
            message: ChatMessage::assistant("done"),
            tool_calls: Vec::new(),
        },
    ]));
    let mut registry = ToolRegistry::new();
    register_default_workflow_tools(&mut registry).unwrap();
    let handler =
        SupportBotHandler::new("support", test_config(), llm, Arc::new(registry), "system");

    let effects = handle_thread(&handler, thread("users", "help"))
        .await
        .unwrap();

    assert!(effects.iter().any(|effect| {
        matches!(
            effect,
            ThreadEffect::Reply { message, .. } if message == "Please send request id"
        )
    }));
    assert!(!effects.iter().any(|effect| {
        matches!(
            effect,
            ThreadEffect::Reply { message, .. }
                if message.contains("private reasoning") || message.contains("<thinking>")
        )
    }));
}

#[tokio::test]
async fn notify_engineer_uses_linked_engineer_thread_reply_effect() {
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "notify_engineer".to_string(),
        arguments: json!({ "message": "need help" }),
    };
    let llm = Arc::new(SequenceLlm::new(vec![
        LlmResponse {
            message: ChatMessage {
                role: ChatRole::Assistant,
                content: None,
                name: None,
                tool_call_id: None,
                tool_calls: vec![call.clone()],
            },
            tool_calls: vec![call],
        },
        LlmResponse {
            message: ChatMessage::assistant("sent"),
            tool_calls: Vec::new(),
        },
    ]));
    let mut registry = ToolRegistry::new();
    register_default_workflow_tools(&mut registry).unwrap();
    let handler =
        SupportBotHandler::new("support", test_config(), llm, Arc::new(registry), "system");

    let effects = handle_thread(&handler, thread("users", "help"))
        .await
        .unwrap();

    assert!(effects.iter().any(|effect| {
        matches!(
            effect,
            ThreadEffect::Reply {
                message, metadata, ..
            }
                if message == "need help"
                    && metadata["support_bot"]["kind"] == "engineer_notification"
        )
    }));
}

#[tokio::test]
async fn tool_loop_limit_replies_with_generic_error_and_stops_thread() {
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "missing_tool".to_string(),
        arguments: json!({}),
    };
    let llm = Arc::new(SequenceLlm::new(vec![LlmResponse {
        message: ChatMessage {
            role: ChatRole::Assistant,
            content: None,
            name: None,
            tool_call_id: None,
            tool_calls: vec![call.clone()],
        },
        tool_calls: vec![call],
    }]));
    let mut config = test_config();
    config.limits.max_tool_rounds = 1;

    let handler = SupportBotHandler::new(
        "support",
        config,
        llm,
        Arc::new(ToolRegistry::new()),
        "system",
    );

    let effects = handle_thread(&handler, thread("users", "help"))
        .await
        .unwrap();

    assert!(effects.iter().any(|effect| {
        matches!(
            effect,
            ThreadEffect::Reply {
                message, metadata, ..
            }
                if message.contains("internal issue")
                    && metadata["support_bot"]["kind"] == "tool_loop_limit"
        )
    }));
    let state_meta = effects
        .iter()
        .find_map(|effect| match effect {
            ThreadEffect::SetThreadMetadata { metadata, .. } => Some(metadata),
            _ => None,
        })
        .expect("state metadata must exist");
    assert_eq!(state_meta[STATE_KEY]["status"], "stopped");
}

#[tokio::test]
async fn finished_user_thread_metadata_skips_workflow() {
    let llm = Arc::new(SequenceLlm::new(vec![LlmResponse {
        message: ChatMessage::assistant("should not run"),
        tool_calls: Vec::new(),
    }]));
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        llm.clone(),
        Arc::new(ToolRegistry::new()),
        "system",
    );
    let mut thread = thread("users", "new reply after finish");
    thread.info.metadata = store_state(
        &json!({}),
        &SupportThreadState {
            status: SupportThreadStatus::Finished,
            finished_summary: Some("done".to_string()),
            ..SupportThreadState::default()
        },
    )
    .unwrap();

    let effects = handle_thread(&handler, thread).await.unwrap();

    assert_eq!(llm.requests().len(), 0);
    assert!(matches!(effects.as_slice(), [ThreadEffect::Noop]));
}

#[tokio::test]
async fn stopped_user_thread_metadata_skips_workflow() {
    let llm = Arc::new(SequenceLlm::new(vec![LlmResponse {
        message: ChatMessage::assistant("should not run"),
        tool_calls: Vec::new(),
    }]));
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        llm.clone(),
        Arc::new(ToolRegistry::new()),
        "system",
    );
    let mut thread = thread("users", "new reply after stop");
    thread.info.metadata = store_state(
        &json!({}),
        &SupportThreadState {
            status: SupportThreadStatus::Stopped,
            ..SupportThreadState::default()
        },
    )
    .unwrap();

    let effects = handle_thread(&handler, thread).await.unwrap();

    assert_eq!(llm.requests().len(), 0);
    assert!(matches!(effects.as_slice(), [ThreadEffect::Noop]));
}

#[tokio::test]
async fn finish_request_persists_finished_state_without_mark_resolved() {
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "finish_request".to_string(),
        arguments: json!({ "summary": "resolved by cache flush" }),
    };
    let llm = Arc::new(SequenceLlm::new(vec![
        LlmResponse {
            message: ChatMessage {
                role: ChatRole::Assistant,
                content: None,
                name: None,
                tool_call_id: None,
                tool_calls: vec![call.clone()],
            },
            tool_calls: vec![call],
        },
        LlmResponse {
            message: ChatMessage::assistant("should not run"),
            tool_calls: Vec::new(),
        },
    ]));
    let mut registry = ToolRegistry::new();
    register_default_workflow_tools(&mut registry).unwrap();
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        llm.clone(),
        Arc::new(registry),
        "system",
    );

    let effects = handle_thread(&handler, thread("users", "help"))
        .await
        .unwrap();

    assert_eq!(llm.requests().len(), 1);

    effects
        .iter()
        .position(|effect| matches!(effect, ThreadEffect::SetThreadMetadata { .. }))
        .expect("SetThreadMetadata must be emitted");
    let state_meta = effects
        .iter()
        .find_map(|effect| match effect {
            ThreadEffect::SetThreadMetadata { metadata, .. } => Some(metadata),
            _ => None,
        })
        .expect("state metadata must exist");
    assert_eq!(state_meta[STATE_KEY]["status"], "finished");
    assert_eq!(
        state_meta[STATE_KEY]["finished_summary"],
        "resolved by cache flush"
    );
}

#[tokio::test]
async fn finish_request_persists_finished_state_and_queues_status_notification() {
    let call = ToolCall {
        id: "call-1".to_string(),
        name: "finish_request".to_string(),
        arguments: json!({ "summary": "resolved by cache flush" }),
    };
    let llm = Arc::new(SequenceLlm::new(vec![LlmResponse {
        message: ChatMessage {
            role: ChatRole::Assistant,
            content: None,
            name: None,
            tool_call_id: None,
            tool_calls: vec![call.clone()],
        },
        tool_calls: vec![call],
    }]));
    let mut registry = ToolRegistry::new();
    register_default_workflow_tools(&mut registry).unwrap();
    let mut config = test_config();
    config.engineer_notifications.channel_id = "engineers".to_string();
    let handler = SupportBotHandler::new("support", config, llm, Arc::new(registry), "system");

    let mut thread = thread("users", "help");
    thread.info.metadata = store_state(&json!({}), &SupportThreadState::default()).unwrap();
    let (ctx, _server) = context_with_snapshot_and_links(
        &thread,
        vec![ThreadLink {
            source_thread_id: thread.info.thread_id.clone(),
            link_kind: ENGINEER_LINK_KIND.to_string(),
            target_thread_id: "engineer-thread".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }],
    )
    .await;

    let effects = handler
        .handle(&invocation_for(&thread), &ctx)
        .await
        .unwrap();

    let state_meta = effects
        .iter()
        .find_map(|effect| match effect {
            ThreadEffect::SetThreadMetadata { metadata, .. } => Some(metadata),
            _ => None,
        })
        .expect("state metadata must exist");
    assert_eq!(state_meta[STATE_KEY]["status"], "finished");

    let trace_meta = effects
        .iter()
        .find_map(|effect| match effect {
            ThreadEffect::SetMessageMetadata { metadata, .. } => Some(metadata),
            _ => None,
        })
        .expect("trace metadata must exist");
    assert!(trace_meta[STATE_KEY][TRACE_KEY]
        .to_string()
        .contains("notification_status"));
    assert!(trace_meta[STATE_KEY][TRACE_KEY]
        .to_string()
        .contains("queued"));
    assert!(effects.iter().any(|effect| {
        matches!(
            effect,
            ThreadEffect::Reply {
            target: ThreadTarget::LinkedThreads { link_kind },
                metadata,
                ..
            } if link_kind == ENGINEER_LINK_KIND
                && metadata[STATE_KEY]["kind"] == "status_update"
        )
    }));
}

#[test]
fn state_is_stored_under_support_bot_key() {
    let metadata = json!({ "other": true });
    let stored = store_state(&metadata, &SupportThreadState::default()).unwrap();

    assert_eq!(stored["other"], true);
    assert_eq!(stored[STATE_KEY]["version"], 1);
}

#[test]
fn tool_registry_specs_are_still_empty_by_default() {
    let registry = ToolRegistry::new();
    let specs: Vec<ToolSpec> = registry.specs();

    assert!(specs.is_empty());
}

#[test]
fn build_llm_messages_keeps_tool_calls_only_in_trace_messages() {
    let handler = SupportBotHandler::new(
        "support",
        test_config(),
        Arc::new(StaticLlm),
        Arc::new(ToolRegistry::new()),
        "system",
    );
    let mut thread = thread("users", "help");
    thread.messages[0].metadata = json!({
        STATE_KEY: {
            TRACE_KEY: [{
                "role": "assistant",
                "content": null,
                "name": null,
                "tool_call_id": null,
                "tool_calls": [{
                    "id": "call-1",
                    "name": "instructions",
                    "arguments": { "id": null }
                }]
            }, {
                "role": "tool",
                "content": "{\"items\":[]}",
                "name": null,
                "tool_call_id": "call-1",
                "tool_calls": []
            }]
        }
    });

    let messages = build_llm_messages(
        &handler.system_prompt,
        &thread,
        &SupportThreadState::default(),
        context().bot_user_id.as_deref(),
    )
    .unwrap();

    let user_msg = messages
        .iter()
        .find(|m| m.role == ChatRole::User)
        .expect("user message should exist");
    assert!(user_msg.tool_calls.is_empty());

    let trace_assistant = messages
        .iter()
        .find(|m| m.role == ChatRole::Assistant && !m.tool_calls.is_empty())
        .expect("assistant tool call message should exist");
    assert_eq!(trace_assistant.tool_calls[0].id, "call-1");
}

#[test]
fn build_llm_messages_orders_thread_messages_by_created_at() {
    let mut thread = thread("users", "second");
    let first_created_at = Utc::now();
    thread.messages[0].post_id = "post-2".to_string();
    thread.messages[0].created_at = first_created_at + chrono::Duration::seconds(1);
    thread.messages.push(ThreadMessage {
        post_id: "post-1".to_string(),
        thread_id: "thread-1".to_string(),
        user_id: "user-1".to_string(),
        message: "first".to_string(),
        root_id: Some("post-1".to_string()),
        parent_post_id: Some("post-1".to_string()),
        props: json!({}),
        metadata: json!({}),
        created_at: first_created_at,
        updated_at: first_created_at,
        is_new: true,
    });

    let messages = build_llm_messages(
        "system",
        &thread,
        &SupportThreadState::default(),
        context().bot_user_id.as_deref(),
    )
    .unwrap();
    let user_contents = messages
        .iter()
        .filter(|message| message.role == ChatRole::User)
        .filter_map(|message| message.content.as_deref())
        .collect::<Vec<_>>();

    assert!(user_contents[0].contains("first"));
    assert!(user_contents[1].contains("second"));
}
