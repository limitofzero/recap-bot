use teloxide::{
    dispatching::dialogue::GetChatId,
    payloads::SendMessageSetters,
    requests::Requester,
    types::{Message, ParseMode},
    Bot,
};

use crate::{app::AppState, services};

pub async fn handle(bot: Bot, msg: Message, state: AppState) -> Result<(), teloxide::RequestError> {
    let chat_id = msg.chat_id();
    if let Some(chat_id) = chat_id {
        if let Ok(response) = services::statistics::top_members(&state.pool, chat_id.0)
            .await
            .inspect_err(|err| {
                log::error!("err when handle top_members: {}", err);
            })
        {
            bot.send_message(chat_id, response)
                .parse_mode(ParseMode::Html)
                .await?;
        }
    }

    Ok(())
}
