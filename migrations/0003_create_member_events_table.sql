CREATE TABLE chats (
    id BIGINT PRIMARY KEY,
    title TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

ALTER TABLE messages ADD CONSTRAINT fk_chat FOREIGN KEY (chat_id) REFERENCES chats(id);

CREATE TABLE member_events (
    id BIGINT PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES chats(id),
    user_id BIGINT NOT NULL,
    actor_id BIGINT,
    event_type TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
