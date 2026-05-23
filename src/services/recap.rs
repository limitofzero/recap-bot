use std::collections::HashMap;
use std::sync::Arc;

use sqlx::PgPool;

use crate::infra::rate_limiter::RateLimiter;
use crate::repositories::messages::StoredMessage;
use crate::{errors::AppError, infra::ai_client::AiClient, repositories};

fn display_name(msg: &StoredMessage) -> String {
    if let Some(u) = &msg.username {
        format!("@{}", u)
    } else if !msg.first_name.trim().is_empty() && msg.first_name != "Unknown" {
        msg.first_name.clone()
    } else {
        format!("user_{}", msg.user_id)
    }
}

fn build_name_map(messages: &[StoredMessage]) -> HashMap<i64, String> {
    let mut map: HashMap<i64, String> = HashMap::new();
    for m in messages {
        let candidate = display_name(m);
        let candidate_is_handle = candidate.starts_with('@');
        match map.get(&m.user_id) {
            Some(existing) if existing.starts_with('@') => {}
            Some(_) if !candidate_is_handle => {}
            _ => {
                map.insert(m.user_id, candidate);
            }
        }
    }
    map
}

fn format_for_llm(messages: &[StoredMessage]) -> String {
    let names = build_name_map(messages);

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
            let name = names.get(&m.user_id).map(String::as_str).unwrap_or("user");
            Some(format!("--- {} | {} ---\n{}", name, time, trimmed))
        })
        .collect::<Vec<_>>()
        .join("\n\n")
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

    let result = ai_client
        .make_request(system_prompt, &formatted, false)
        .await?;
    log::debug!("ai response: {}", result);

    Ok(result)
}
