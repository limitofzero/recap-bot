use sqlx::PgPool;

use crate::{
    errors::AppError,
    formatters,
    infra::ai_client::AiClient,
    repositories::{self, messages::StoredUserMessage},
};

pub async fn build_recap(
    pool: &PgPool,
    ai_client: &AiClient,
    sys_prompt: &str,
    chat_id: i64,
    normalized_username: &str,
) -> Result<String, AppError> {
    let msgs =
        repositories::messages::get_last_by_user(pool, chat_id, normalized_username, 50).await?;

    let user_prompt = format_for_llm(normalized_username, &msgs);

    let recap = ai_client
        .make_request(sys_prompt, &user_prompt, false)
        .await?;

    Ok(recap)
}

fn format_for_llm(username: &str, messages: &[StoredUserMessage]) -> String {
    let mut out = format!("=== Messages from @{} ===\n\n", username);

    for msg in messages {
        if let Some(formatted_msg) =
            formatters::ai_summary::format_message(&msg.text, &msg.created_at)
        {
            out.push_str(&formatted_msg);
            out.push_str("\n\n");
        }
    }

    out
}
