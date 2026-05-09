use std::collections::VecDeque;
use std::sync::Arc;

use futures_util::stream::{self, BoxStream, StreamExt};
use mattermost_api::apis::configuration::Configuration;
use mattermost_api::apis::posts_api;
use mattermost_api::models;

use crate::error::ThreadBotError;
use crate::types::ChannelCheckpoint;

const PAGE_SIZE: i32 = 200;

struct ChannelMessagesState {
    config: Arc<Configuration>,
    channel_id: String,
    after_post_id: String,
    buffer: VecDeque<models::Post>,
    exhausted: bool,
}

/// Stream channel posts after a persisted checkpoint.
///
/// Uses Mattermost's `after` cursor instead of `since`; the API documents
/// `since` as non-consecutive, so it is not suitable for reconciliation.
pub(crate) fn channel_messages_after(
    config: Arc<Configuration>,
    checkpoint: &ChannelCheckpoint,
) -> BoxStream<'static, Result<models::Post, ThreadBotError>> {
    let state = ChannelMessagesState {
        config,
        channel_id: checkpoint.channel_id.clone(),
        after_post_id: checkpoint.last_seen_post_id.clone(),
        buffer: VecDeque::new(),
        exhausted: false,
    };

    stream::unfold(state, |mut state| async move {
        loop {
            if let Some(post) = state.buffer.pop_front() {
                state.after_post_id = post.id.clone();
                return Some((Ok(post), state));
            }

            if state.exhausted {
                return None;
            }

            if let Err(error) = fetch_next_page(&mut state).await {
                state.exhausted = true;
                return Some((Err(error), state));
            }
        }
    })
    .boxed()
}

async fn fetch_next_page(state: &mut ChannelMessagesState) -> Result<(), ThreadBotError> {
    let post_list = posts_api::get_posts_for_channel(
        &state.config,
        &state.channel_id,
        None,
        Some(PAGE_SIZE),
        None,
        None,
        Some(&state.after_post_id),
        None,
    )
    .await
    .map_err(ThreadBotError::mattermost_api)?;

    let page_len = post_list.order.as_ref().map_or(0, Vec::len);
    if let (Some(order), Some(posts)) = (post_list.order, post_list.posts) {
        let next_after_post_id = order.last().cloned();
        state.buffer = order
            .into_iter()
            .filter_map(|post_id| posts.get(&post_id).cloned())
            .collect();
        if state.buffer.is_empty() {
            if let Some(post_id) = next_after_post_id {
                state.after_post_id = post_id;
            }
        }
    }

    if page_len == 0 || !post_list.has_next.unwrap_or(false) {
        state.exhausted = true;
    }

    Ok(())
}

pub(crate) async fn latest_channel_post(
    config: &Configuration,
    channel_id: &str,
) -> Result<Option<models::Post>, ThreadBotError> {
    let post_list = posts_api::get_posts_for_channel(
        config,
        channel_id,
        Some(0),
        Some(1),
        None,
        None,
        None,
        None,
    )
    .await
    .map_err(ThreadBotError::mattermost_api)?;

    let Some(order) = post_list.order else {
        return Ok(None);
    };
    let Some(posts) = post_list.posts else {
        return Ok(None);
    };
    let Some(post_id) = order.first() else {
        return Ok(None);
    };

    Ok(posts.get(post_id).cloned())
}
