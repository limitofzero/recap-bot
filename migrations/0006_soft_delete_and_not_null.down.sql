-- 0005_soft_delete_and_not_null.down.sql
DROP INDEX IF EXISTS idx_chat_members_active;
ALTER TABLE chat_members DROP COLUMN IF EXISTS deleted_at;

ALTER TABLE messages DROP CONSTRAINT IF EXISTS fk_messages_users;
ALTER TABLE messages ALTER COLUMN user_id DROP NOT NULL;
ALTER TABLE messages
    ADD CONSTRAINT fk_messages_users
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL;