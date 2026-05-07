#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Save message error: {0}")]
    SaveMessage(String),

    #[error("Update message error: {0}")]
    UpdateMessage(String),

    #[error("Parse message error: {0}")]
    ParseMessage(String),

    #[error("Empty chat id for message({0})")]
    EmptyChatId(i64),

    #[error("Empty user id for message({0}), chat_id: {1}")]
    EmptyUserId(i64, i64),

    #[error("Error when isert or update chat, chat_id: {0}, error: {1}")]
    InsertChatError(i64, String),
}
