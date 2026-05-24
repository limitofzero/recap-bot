use std::time::Duration;

pub fn message_received(kind: MessageKind) {
    ::metrics::counter!("bot_messages_received_total", "kind" => kind.as_str()).increment(1);
}

/// Record the duration of a database query, grouped by logical operation.
pub fn db_query(op: DbOp, elapsed: Duration) {
    ::metrics::histogram!("bot_db_query_seconds", "operation" => op.as_str())
        .record(elapsed.as_secs_f64());
}

pub fn llm_tokens(kind: TokenKind, count: u64) {
    ::metrics::counter!("bot_llm_tokens_total", "kind" => kind.as_str()).increment(count);
}

#[derive(Debug, Clone, Copy)]
pub enum MessageKind {
    Text,
    Command,
    Edit,
}

impl MessageKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Command => "command",
            Self::Edit => "edit",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DbOp {
    InsertMessage,
    SelectMessage,
    InsertChat,
    UpsertChatMember,
    UpsertUser,
    SelectTopMembers,
    SelectRecentUsersMessages,
}

impl DbOp {
    fn as_str(self) -> &'static str {
        match self {
            Self::InsertMessage => "insert_message",
            Self::SelectMessage => "select_message",
            Self::InsertChat => "insert_chat",
            Self::UpsertChatMember => "upsert_chat_member",
            Self::UpsertUser => "upsert_user",
            Self::SelectTopMembers => "select_top_members",
            Self::SelectRecentUsersMessages => "select_recent_user_messages",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TokenKind {
    Prompt,
    Completion,
}

impl TokenKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Prompt => "prompt",
            Self::Completion => "completion",
        }
    }
}
