CREATE TABLE threads (
    thread_id TEXT PRIMARY KEY,
    root_post_id TEXT NOT NULL UNIQUE,
    channel_id TEXT NOT NULL,
    creator_user_id TEXT NOT NULL,
    status TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    last_seen_post_id TEXT,
    last_seen_post_at TIMESTAMPTZ,
    last_processed_post_id TEXT,
    last_processed_post_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_threads_channel_id ON threads(channel_id);
CREATE INDEX idx_threads_status ON threads(status);
CREATE INDEX idx_threads_updated_at ON threads(updated_at DESC);
CREATE INDEX idx_threads_metadata_gin ON threads USING GIN(metadata);

CREATE TABLE thread_messages (
    post_id TEXT PRIMARY KEY,
    thread_id TEXT NOT NULL REFERENCES threads(thread_id) ON DELETE CASCADE,
    user_id TEXT NOT NULL,
    is_bot_message BOOLEAN NOT NULL DEFAULT FALSE,
    root_id TEXT,
    parent_post_id TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    post_created_at TIMESTAMPTZ NOT NULL,
    post_updated_at TIMESTAMPTZ,
    post_deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_thread_messages_thread_id_post_created_at
    ON thread_messages(thread_id, post_created_at ASC);
CREATE INDEX idx_thread_messages_user_id ON thread_messages(user_id);
CREATE INDEX idx_thread_messages_metadata_gin ON thread_messages USING GIN(metadata);
CREATE INDEX idx_thread_messages_is_bot ON thread_messages(is_bot_message)
    WHERE is_bot_message = TRUE;

CREATE TABLE thread_reactions (
    id BIGSERIAL PRIMARY KEY,
    thread_id TEXT NOT NULL REFERENCES threads(thread_id) ON DELETE CASCADE,
    post_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    emoji_name TEXT NOT NULL,
    action TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_thread_reactions_thread_id ON thread_reactions(thread_id);
CREATE INDEX idx_thread_reactions_post_id ON thread_reactions(post_id);
CREATE INDEX idx_thread_reactions_emoji_name ON thread_reactions(emoji_name);

CREATE TABLE channel_checkpoints (
    channel_id TEXT PRIMARY KEY,
    last_seen_post_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE thread_runs (
    id BIGSERIAL PRIMARY KEY,
    thread_id TEXT NOT NULL REFERENCES threads(thread_id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    trigger TEXT NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    finished_at TIMESTAMPTZ,
    error TEXT
);

CREATE INDEX idx_thread_runs_thread_id_started_at
    ON thread_runs(thread_id, started_at DESC);
