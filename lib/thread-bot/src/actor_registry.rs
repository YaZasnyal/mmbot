use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{mpsc, Mutex, RwLock};

use mattermost_api::apis::configuration::Configuration;

use crate::actor::{thread_actor, ActorCtx, ThreadCommand};
use crate::error::ThreadBotError;
use crate::handler::{ThreadContext, ThreadHandler};
use crate::metrics::ThreadBotMetricsHandle;
use crate::store::ThreadStore;

#[derive(Clone)]
pub(crate) struct ActorRegistry {
    inner: Arc<Mutex<HashMap<String, mpsc::Sender<ThreadCommand>>>>,
}

pub(crate) struct ActorSpawnConfig<H: ThreadHandler> {
    pub handler: Arc<H>,
    pub store: Arc<dyn ThreadStore>,
    pub mm_config: Arc<Configuration>,
    pub debounce: Duration,
    pub channel_buffer: usize,
    pub actor_idle_timeout: Duration,
    pub thread_id: String,
    pub bot_user_id: Arc<RwLock<Option<String>>>,
    pub metrics: ThreadBotMetricsHandle,
}

impl ActorRegistry {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub(crate) async fn send_or_spawn<H>(
        &self,
        command: ThreadCommand,
        config: ActorSpawnConfig<H>,
    ) -> Result<(), ThreadBotError>
    where
        H: ThreadHandler,
    {
        let thread_id = config.thread_id.clone();
        let tx = self.ensure_or_spawn(config).await;

        if tx.send(command).await.is_err() {
            self.remove(&thread_id).await;
            return Err(actor_channel_closed(&thread_id));
        }

        Ok(())
    }

    pub(crate) async fn send_if_active(
        &self,
        thread_id: &str,
        command: ThreadCommand,
    ) -> Result<bool, ThreadBotError> {
        let tx = {
            let mut actors = self.inner.lock().await;
            let Some(tx) = actors.get(thread_id) else {
                return Ok(false);
            };

            if tx.is_closed() {
                actors.remove(thread_id);
                return Ok(false);
            }

            tx.clone()
        };

        if tx.send(command).await.is_err() {
            self.remove(thread_id).await;
            return Ok(false);
        }

        Ok(true)
    }

    pub(crate) async fn send_shutdown_to_all(&self) -> usize {
        let senders = {
            let actors = self.inner.lock().await;
            actors.values().cloned().collect::<Vec<_>>()
        };
        let actor_count = senders.len();

        for tx in senders {
            let _ = tx.send(ThreadCommand::Shutdown).await;
        }

        actor_count
    }

    async fn ensure_or_spawn<H>(&self, config: ActorSpawnConfig<H>) -> mpsc::Sender<ThreadCommand>
    where
        H: ThreadHandler,
    {
        let mut actors = self.inner.lock().await;

        if let Some(tx) = actors.get(&config.thread_id) {
            if !tx.is_closed() {
                return tx.clone();
            }
            actors.remove(&config.thread_id);
        }

        let (tx, rx) = mpsc::channel(config.channel_buffer);
        actors.insert(config.thread_id.clone(), tx.clone());
        drop(actors);

        self.spawn_actor(config, rx);

        tx
    }

    fn spawn_actor<H>(&self, config: ActorSpawnConfig<H>, rx: mpsc::Receiver<ThreadCommand>)
    where
        H: ThreadHandler,
    {
        let registry = self.clone();
        let ActorSpawnConfig {
            handler,
            store,
            mm_config,
            debounce,
            channel_buffer: _,
            actor_idle_timeout,
            thread_id,
            bot_user_id,
            metrics,
        } = config;
        let plugin_id = handler.id();

        tokio::spawn(async move {
            let mut rx = rx;
            let bot_user_id_snapshot = bot_user_id.read().await.clone();
            let actor_ctx = ActorCtx {
                handler,
                store: Arc::clone(&store),
                ctx: ThreadContext {
                    config: Arc::clone(&mm_config),
                    store,
                    plugin_id,
                    bot_user_id: bot_user_id_snapshot,
                },
                mm_config,
                debounce,
                actor_idle_timeout,
                thread_id: thread_id.clone(),
                bot_user_id,
                metrics,
            };

            loop {
                thread_actor(&mut rx, &actor_ctx).await;

                if registry.should_continue_after_finish(&thread_id, &rx).await {
                    continue;
                }

                break;
            }

            tracing::info!(thread_id = %thread_id, "finished actor for thread");
        });
    }

    async fn should_continue_after_finish(
        &self,
        thread_id: &str,
        rx: &mpsc::Receiver<ThreadCommand>,
    ) -> bool {
        let mut actors = self.inner.lock().await;
        let Some(current) = actors.get(thread_id) else {
            tracing::debug!(
                thread_id = %thread_id,
                "Actor registry entry is already absent"
            );
            return false;
        };

        let sender_count = current.strong_count();
        if sender_count >= 2 || !rx.is_empty() {
            tracing::info!(
                thread_id = %thread_id,
                queued_commands = rx.len(),
                sender_count,
                "Actor received or may receive commands during teardown, continuing"
            );
            return true;
        }

        actors.remove(thread_id);
        false
    }

    async fn remove(&self, thread_id: &str) {
        self.inner.lock().await.remove(thread_id);
    }
}

fn actor_channel_closed(thread_id: &str) -> ThreadBotError {
    ThreadBotError::InvalidState(format!("actor channel closed for thread {thread_id}"))
}
