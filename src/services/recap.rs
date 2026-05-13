use std::sync::Arc;

use sqlx::PgPool;

use crate::repositories::messages::StoredMessage;
use crate::{errors::AppError, repositories, services::ai_client::AiClient};

fn display_name(msg: &StoredMessage) -> String {
    if let Some(u) = &msg.username {
        format!("@{}", u)
    } else if !msg.first_name.trim().is_empty() && msg.first_name != "Unknown" {
        msg.first_name.clone()
    } else {
        format!("user_{}", msg.user_id)
    }
}

fn format_for_llm(messages: &[StoredMessage]) -> String {
    messages
        .iter()
        .rev()
        .filter_map(|m| {
            let text = m.text.as_ref()?;
            let trimmed = text.trim();
            if trimmed.is_empty() || trimmed.starts_with('/') {
                return None;
            }
            let time = m.created_at.format("%H:%M");
            Some(format!("[{}] {}: {}", time, display_name(m), trimmed))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub async fn build_recap(
    pool: &PgPool,
    ai_client: Arc<AiClient>,
    system_prompt: &str,
    chat_id: i64,
    count: usize,
) -> Result<String, AppError> {
    let messages = repositories::messages::get_last(pool, chat_id, count as i64).await?;

    let formatted = format_for_llm(&messages);

    if formatted.is_empty() {
        return Ok("Недостаточно сообщений для саммари.".to_string());
    }

    log::debug!("formatted prompt:\n{}", formatted);

    let result = ai_client.make_request(system_prompt, &formatted).await?;
    log::debug!("ai response: {}", result);

    Ok(result)
}
