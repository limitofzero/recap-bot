DELETE FROM messages WHERE user_id IS NULL;

ALTER TABLE messages DROP CONSTRAINT IF EXISTS fk_messages_users;
ALTER TABLE messages ALTER COLUMN user_id SET NOT NULL;
ALTER TABLE messages
    ADD CONSTRAINT fk_messages_users
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE RESTRICT;

ALTER TABLE chat_members ADD COLUMN deleted_at TIMESTAMPTZ;

CREATE INDEX idx_chat_members_active ON chat_members(chat_id) WHERE deleted_at IS NULL;
