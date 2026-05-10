use async_trait::async_trait;
use chrono::{DateTime, Utc};
use thread_bot::{
    AppendReaction, ChannelCheckpoint, ThreadBotError, ThreadLink, ThreadMessageRecord,
    ThreadReaction, ThreadRecord, ThreadStore, UpsertThread, UpsertThreadLink, UpsertThreadMessage,
};

pub struct PanicStore;

#[async_trait]
impl ThreadStore for PanicStore {
    async fn upsert_thread(&self, _input: UpsertThread) -> Result<ThreadRecord, ThreadBotError> {
        panic!("store should not be used")
    }

    async fn get_thread(&self, _thread_id: &str) -> Result<Option<ThreadRecord>, ThreadBotError> {
        panic!("store should not be used")
    }

    async fn get_thread_by_post(
        &self,
        _post_id: &str,
    ) -> Result<Option<ThreadRecord>, ThreadBotError> {
        panic!("store should not be used")
    }

    async fn list_threads(
        &self,
        _updated_after: Option<DateTime<Utc>>,
        _updated_before: Option<DateTime<Utc>>,
    ) -> Result<Vec<ThreadRecord>, ThreadBotError> {
        panic!("store should not be used")
    }

    async fn set_thread_metadata(
        &self,
        _thread_id: &str,
        _metadata: serde_json::Value,
    ) -> Result<(), ThreadBotError> {
        panic!("store should not be used")
    }

    async fn upsert_thread_link(
        &self,
        _input: UpsertThreadLink,
    ) -> Result<ThreadLink, ThreadBotError> {
        panic!("store should not be used")
    }

    async fn get_thread_link(
        &self,
        _source_thread_id: &str,
        _link_kind: &str,
    ) -> Result<Option<ThreadLink>, ThreadBotError> {
        panic!("store should not be used")
    }

    async fn list_thread_links(
        &self,
        _source_thread_id: &str,
    ) -> Result<Vec<ThreadLink>, ThreadBotError> {
        panic!("store should not be used")
    }

    async fn list_reverse_thread_links(
        &self,
        _target_thread_id: &str,
    ) -> Result<Vec<ThreadLink>, ThreadBotError> {
        panic!("store should not be used")
    }

    async fn update_thread_seen(
        &self,
        _thread_id: &str,
        _post_id: &str,
        _seen_at: DateTime<Utc>,
    ) -> Result<(), ThreadBotError> {
        panic!("store should not be used")
    }

    async fn update_thread_processed(
        &self,
        _thread_id: &str,
        _post_id: &str,
        _processed_at: DateTime<Utc>,
    ) -> Result<(), ThreadBotError> {
        panic!("store should not be used")
    }

    async fn upsert_message(
        &self,
        _input: UpsertThreadMessage,
    ) -> Result<ThreadMessageRecord, ThreadBotError> {
        panic!("store should not be used")
    }

    async fn get_message(
        &self,
        _post_id: &str,
    ) -> Result<Option<ThreadMessageRecord>, ThreadBotError> {
        panic!("store should not be used")
    }

    async fn list_thread_messages(
        &self,
        _thread_id: &str,
    ) -> Result<Vec<ThreadMessageRecord>, ThreadBotError> {
        panic!("store should not be used")
    }

    async fn set_message_metadata(
        &self,
        _post_id: &str,
        _metadata: serde_json::Value,
    ) -> Result<(), ThreadBotError> {
        panic!("store should not be used")
    }

    async fn append_reaction(&self, _input: AppendReaction) -> Result<(), ThreadBotError> {
        panic!("store should not be used")
    }

    async fn list_thread_reactions(
        &self,
        _thread_id: &str,
    ) -> Result<Vec<ThreadReaction>, ThreadBotError> {
        panic!("store should not be used")
    }

    async fn list_channel_checkpoints(&self) -> Result<Vec<ChannelCheckpoint>, ThreadBotError> {
        panic!("store should not be used")
    }

    async fn upsert_channel_checkpoint(
        &self,
        _channel_id: &str,
        _last_seen_post_id: &str,
        _last_seen_post_at: DateTime<Utc>,
    ) -> Result<(), ThreadBotError> {
        panic!("store should not be used")
    }

    async fn advance_channel_checkpoint(
        &self,
        _channel_id: &str,
        _last_seen_post_id: &str,
        _last_seen_post_at: DateTime<Utc>,
    ) -> Result<(), ThreadBotError> {
        panic!("store should not be used")
    }

    async fn set_all_channels_not_reconciled(&self) -> Result<(), ThreadBotError> {
        panic!("store should not be used")
    }

    async fn set_channel_reconciled(&self, _channel_id: &str) -> Result<(), ThreadBotError> {
        panic!("store should not be used")
    }
}
