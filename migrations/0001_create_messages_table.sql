CREATE TABLE messages (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL,
    message_id BIGINT NOT NULL,
    media_group_id TEXT,
    media_url TEXT,
    user_id BIGINT,
    file_id TEXT,
    content_type TEXT NOT NULL DEFAULT 'text',
    payload JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    edited_at TIMESTAMPTZ,
    UNIQUE(chat_id, message_id)
)