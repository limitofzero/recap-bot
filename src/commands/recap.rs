use std::sync::Arc;

use teloxide::{dispatching::dialogue::GetChatId, prelude::*};

use crate::{
    app::AppState,
    domain::consts::{DEFAULT_MSG_RECAP, MAX_MSG_RECAP},
    services,
};

pub async fn handle(
    bot: Bot,
    msg: Message,
    args: String,
    state: AppState,
) -> Result<(), teloxide::RequestError> {
    let count = parse_count(args);
    bot.send_message(msg.chat.id, "In progress...".to_string())
        .await?;

    let chat_id = msg.chat_id().map(|chat_id| chat_id.0).unwrap_or(0);
    if chat_id == 0 {
        log::error!("chat_id is empty for message_id: {}", msg.id.0);
        bot.send_message(msg.chat.id, "Shit is happened!".to_string())
            .await?;
    } else {
        let ai_client = Arc::clone(&state.ai_client);
        let response = services::recap::build_recap(
            &state.pool,
            ai_client,
            &state.ai_recap_system_prompt,
            chat_id,
            count,
        )
        .await;

        match response {
            Ok(response) => {
                bot.send_message(msg.chat.id, response).await?;
            }
            Err(err) => {
                log::error!("error: {}", err);
                bot.send_message(msg.chat.id, "Shit is happened!".to_string())
                    .await?;
            }
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
