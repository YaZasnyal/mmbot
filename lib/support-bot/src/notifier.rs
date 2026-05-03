use crate::error::Result;
use async_trait::async_trait;
use thread_bot::Thread;

#[async_trait]
pub trait SupportNotifier: Send + Sync {
    async fn send_user_message(&self, thread: &Thread, message: String) -> Result<()>;

    async fn notify_engineer(&self, thread: &Thread, message: String) -> Result<()>;
}
