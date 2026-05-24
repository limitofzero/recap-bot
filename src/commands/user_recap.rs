use teloxide::requests::Requester;
use teloxide::{types::Message, Bot};

use crate::app::AppState;
use crate::domain::promts::Prompt;
use crate::handlers::rate_limit;
use crate::{services, validators};

pub async fn handle(
    bot: Bot,
    msg: Message,
    maybe_username: String,
    state: AppState,
) -> Result<(), teloxide::RequestError> {
    if !validators::username::is_valid(&maybe_username) {
        bot.send_message(msg.chat.id, "Username is invalid").await?;
        return Ok(());
    }

    if !rate_limit::allowed(&bot, &msg, &state.rate_limiter).await? {
        return Ok(());
    }

    let sys_prompt = state
        .get_promt_or_error(Prompt::UserRecap)
        .map_err(|err| teloxide::ApiError::Unknown(err.to_string()))?;

    match services::user_recap::build_recap(
        &state.pool,
        &state.ai_client,
        sys_prompt,
        msg.chat.id.0,
        validators::username::normalize(&maybe_username),
    )
    .await
    {
        Ok(recap) => {
            bot.send_message(msg.chat.id, recap).await?;
        }
        Err(err) => {
            log::error!(
                "error when build user recap, chat_id({}), username({}): {:?}",
                msg.chat.id.0,
                maybe_username,
                err
            );
            bot.send_message(msg.chat.id, "Shit happened!").await?;
        }
    }

    Ok(())
}
