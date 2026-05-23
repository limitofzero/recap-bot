use teloxide::prelude::*;

use crate::{
    app::AppState,
    domain::consts::{DEFAULT_MSG_RECAP, MAX_MSG_RECAP},
    domain::promts::Prompt,
    handlers::rate_limit,
    services,
};

pub async fn handle(
    bot: Bot,
    msg: Message,
    args: String,
    state: AppState,
) -> Result<(), teloxide::RequestError> {
    if !rate_limit::allowed(&bot, &msg, &state.rate_limiter).await? {
        return Ok(());
    }

    let count = parse_count(args);
    bot.send_message(msg.chat.id, "In progress...").await?;

    let chat_id = msg.chat.id.0;
    let prompt = state
        .get_promt_or_error(Prompt::Recap)
        .map_err(|err| teloxide::ApiError::Unknown(err.to_string()))?;

    let response =
        services::recap::build_recap(&state.pool, &state.ai_client, prompt, chat_id, count).await;

    match response {
        Ok(response) => {
            bot.send_message(msg.chat.id, response).await?;
        }
        Err(err) => {
            log::error!("error: {}", err);
            bot.send_message(msg.chat.id, "Shit happened!").await?;
        }
    }

    Ok(())
}

fn parse_count(args: String) -> usize {
    if args.trim().is_empty() {
        DEFAULT_MSG_RECAP
    } else {
        args.trim()
            .parse::<usize>()
            .map(|val| val.clamp(1, MAX_MSG_RECAP))
            .unwrap_or(DEFAULT_MSG_RECAP)
    }
}
