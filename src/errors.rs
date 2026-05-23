use crate::domain::promts::Prompt;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Save message error: {0}")]
    SaveMessage(String),

    #[error("Empty chat id for message({0})")]
    EmptyChatId(i64),

    #[error("Error when isert or update chat, chat_id: {0}, error: {1}")]
    InsertChatError(i64, String),

    #[error("Select messages error for chat_id: {0}: {1}")]
    SelectMessagesError(i64, String),

    #[error("Error fetching ai response: {0}")]
    AiResponse(String),

    #[error("Prompt({0}) not found")]
    PromptNotFound(Prompt),

    #[error("Db query error: {0}")]
    DbError(String),
}
