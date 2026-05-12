use std::sync::Arc;

use teloxide::{dispatching::dialogue::GetChatId, prelude::*};

use crate::{app::AppState, domain::{consts::MAX_MSG_RECAP}, services};

pub async fn handle(
    bot: Bot,
    msg: Message,
    count: usize,
    _state: AppState,
) -> Result<(), teloxide::RequestError> {
    if count == 0 || count as usize > MAX_MSG_RECAP {
        bot.send_message(
            msg.chat.id,
            "Count must be between 1 and 500. Usage: /recap 100",
        )
        .await?;
        return Ok(());
    }
    
    bot.send_message(msg.chat.id, format!("Recap of last {} messages — TODO", count))
        .await?;

    let chat_id = msg.chat_id().map(|chat_id| chat_id.0).unwrap_or(0);
    if chat_id == 0 {
        log::error!("chat_id is empty for message_id: {}", msg.id.0);
        bot.send_message(msg.chat.id, format!("Shit is happened!")).await?;
    } else {
        let ai_client = Arc::clone(&_state.ai_client);
        let response = services::recap::build_recap(&_state.pool, ai_client, chat_id, count).await;
        
        match response {
            Ok(response) => {
                bot.send_message(msg.chat.id, response).await?;
            },
            Err(err) => {
                log::error!("error: {}", err.to_string());
                bot.send_message(msg.chat.id, format!("Shit is happened!")).await?;
            }
        }
    }

    Ok(())
}