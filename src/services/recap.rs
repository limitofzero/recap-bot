use std::sync::Arc;

use sqlx::PgPool;

use crate::{errors::AppError, repositories, services::ai_client::{AiClient}};


const SYSTEM_PROMPT: &str = "You are a Telegram chat summarizer. \
    Output a concise digest in the same language as the input. \
    Group related topics into bullet points. Max 300 words. \
    Skip greetings and small talk.";

pub async fn build_recap(pool: &PgPool, ai_client: Arc<AiClient>, chat_id: i64, count: usize) -> Result<String, AppError> {
    let messages = repositories::messages::get_last(pool, chat_id, count as i64)
        .await?;

    let serilized_messages = serde_json::to_string(&messages)
        .map_err(|err| AppError::SerializeMessages(err.to_string()))?;

    let result = ai_client.make_request(SYSTEM_PROMPT, serilized_messages).await?;
    log::debug!("get ai response: {}", result);

    Ok(result)
}