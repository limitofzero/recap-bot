use sqlx::PgPool;

use crate::{errors::AppError, formatters, infra::ai_client::AiClient, repositories};

pub async fn build_recap(
    pool: &PgPool,
    ai_client: &AiClient,
    system_prompt: &str,
    chat_id: i64,
    count: usize,
) -> Result<String, AppError> {
    let messages = repositories::messages::get_last(pool, chat_id, count as i64).await?;

    let formatted = formatters::ai_summary::format_messages_for_llm(&messages);

    if formatted.is_empty() {
        return Ok("Недостаточно сообщений для саммари.".to_string());
    }

    log::debug!("formatted prompt:\n{}", formatted);

    let result = ai_client
        .make_request(system_prompt, &formatted, false)
        .await?;
    log::debug!("ai response: {}", result);

    Ok(result)
}
