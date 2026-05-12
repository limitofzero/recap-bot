CREATE TABLE IF NOT EXISTS users (
    id BIGINT PRIMARY KEY,
    is_bot BOOLEAN DEFAULT FALSE NOT NULL,
    first_name TEXT NOT NULL,
    last_name TEXT,
    username TEXT,
    is_premium BOOLEAN,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO users (id, first_name)
SELECT DISTINCT user_id, 'Unknown'
FROM messages
WHERE user_id IS NOT NULL
ON CONFLICT (id) DO NOTHING;

CREATE INDEX IF NOT EXISTS idx_messages_user_id ON messages(user_id);

ALTER TABLE messages
    ADD CONSTRAINT fk_messages_users
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL;
