use teloxide::dispatching::{HandlerExt, UpdateFilterExt, UpdateHandler};
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

use crate::app::AppState;
use crate::commands;
use crate::domain::commands::Command;

pub fn router() -> UpdateHandler<teloxide::RequestError> {
    Update::filter_message()
        .filter_command::<Command>()
        .endpoint(dispatch)
}

async fn dispatch(
    bot: Bot,
    msg: Message,
    cmd: Command,
    state: AppState,
) -> Result<(), teloxide::RequestError> {
    let result = match cmd {
        Command::Help => bot
            .send_message(msg.chat.id, Command::descriptions().to_string())
            .await
            .map(|_| ()),
        Command::Recap(args) => commands::recap::handle(bot, msg, args, state).await,
    };

    if let Err(err) = result {
        log::error!("command handler error: {}", err);
    }
    Ok(())
}
