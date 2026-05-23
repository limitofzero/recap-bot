use teloxide::prelude::*;

use crate::app::AppState;
use crate::domain::promts::Prompt;
use crate::formatters::username::get_username;
use crate::handlers::rate_limit;
use crate::services;

pub async fn handle(bot: Bot, msg: Message, state: AppState) -> Result<(), teloxide::RequestError> {
    if let Err(err) = services::messages::save(&state.pool, &msg).await {
        log::error!("handle message error: {}", err);
    }

    let sys_prompt = state
        .get_promt_or_error(Prompt::ResponseToUser)
        .map_err(|err| teloxide::ApiError::Unknown(err.to_string()))?;

    let chat_id = msg.chat.id;

    if is_mentioned(&msg, &state.bot.name) || is_reply_to_bot(&msg, state.bot.id) {
        if !rate_limit::allowed(&bot, &msg, &state.rate_limiter).await? {
            return Ok(());
        }

        let Some(from) = msg.from.as_ref() else {
            return Ok(());
        };
        let username = get_username(from);

        let previous_msg = msg.reply_to_message().and_then(|msg| msg.text());
        match services::response_to_user::response(
            &msg,
            &state.pool,
            &state.ai_client,
            sys_prompt,
            &username,
            previous_msg,
        )
        .await
        {
            Ok(response) => {
                bot.send_message(chat_id, response).await?;
            }
            Err(err) => {
                log::error!("reresponse_to_user err: {:?}", err);
                bot.send_message(chat_id, "Shit happened!").await?;
            }
        }
    }

    Ok(())
}

fn is_reply_to_bot(msg: &Message, bot_id: u64) -> bool {
    msg.reply_to_message()
        .and_then(|reply| reply.from.as_ref())
        .map(|user| user.id.0 == bot_id)
        .unwrap_or(false)
}

fn is_mentioned(msg: &Message, bot_name: &str) -> bool {
    let Some(text) = msg.text() else { return false };
    let needle = format!("@{bot_name}");
    text.split_whitespace()
        .any(|word| word.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_') == needle)
}
