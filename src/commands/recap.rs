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
    _state: AppState,
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
        let ai_client = Arc::clone(&_state.ai_client);
        let response = services::recap::build_recap(
            &_state.pool,
            ai_client,
            &_state.ai_recap_system_prompt,
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
    if !args.trim().is_empty() {
        DEFAULT_MSG_RECAP
    } else {
        args.trim()
            .parse()
            .map(coelse_count)
            .unwrap_or(DEFAULT_MSG_RECAP)
    }
}

fn coelse_count(count: usize) -> usize {
    if count > MAX_MSG_RECAP || count == 0 {
        MAX_MSG_RECAP
    } else {
        count
    }
}
