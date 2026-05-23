use sqlx::PgPool;
use teloxide::types::Message;

use crate::{
    errors::AppError,
    formatters,
    infra::ai_client::AiClient,
    repositories::{self, messages::StoredMessage},
};

pub async fn response(
    msg: &Message,
    pool: &PgPool,
    ai_client: &AiClient,
    sys_prompt: &str,
    username: &str,
) -> Result<String, AppError> {
    let chat_id = msg.chat.id.0;
    let recent_msgs = repositories::messages::get_last(pool, chat_id, 50).await?;

    let user_prompt = get_user_prompt(&recent_msgs, msg, username);

    let response = ai_client
        .make_request(sys_prompt, &user_prompt, false)
        .await?;

    Ok(response)
}

fn get_user_prompt(messages: &Vec<StoredMessage>, msg: &Message, username: &str) -> String {
    let chat_summary = formatters::ai_summary::format_messages_for_llm(messages);
    let chat_context_msg = format!(
        "=== Context with last {} messages ===\n\n {}",
        messages.len(),
        chat_summary
    );

    format!(
        "{}\n\nUser({username}) just said to you: {}\n",
        chat_context_msg,
        msg.text().unwrap_or("")
    )
}
