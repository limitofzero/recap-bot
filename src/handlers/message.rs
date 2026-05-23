use teloxide::prelude::*;

use crate::app::AppState;
use crate::domain::promts::Prompt;
use crate::formatters::username::get_username;
use crate::services;

pub async fn handle(bot: Bot, msg: Message, state: AppState) -> Result<(), teloxide::RequestError> {
    if let Err(err) = services::messages::save(&state.pool, &msg).await {
        log::error!("handle message error: {}", err);
    }

    let sys_prompt = state
        .get_promt_or_error(Prompt::ResponseToUser)
        .map_err(|err| teloxide::ApiError::Unknown(err.to_string()))?;

    let chat_id = msg.chat.id;
    // let previous_msg = msg.reply_to_message().as_ref().map(|msg| msg.text())
    if is_mentioned(&msg, &state.bot.name) {
        let Some((user_id, username)) = msg
            .from
            .as_ref()
            .map(|user| (user.id.0, get_username(user)))
        else {
            return Ok(());
        };

        if state.rate_limiter.check(user_id).await.is_err() {
            bot.send_message(msg.chat.id, "Лимит превышен, так что иди нахуй...")
                .await?;
            return Ok(());
        }

        match services::response_to_user::response(
            &msg,
            &state.pool,
            &state.ai_client,
            sys_prompt,
            &username,
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
