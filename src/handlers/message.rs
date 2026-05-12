use teloxide::prelude::*;

use crate::app::AppState;
use crate::services;

pub async fn handle(_: Bot, msg: Message, state: AppState) -> Result<(), teloxide::RequestError> {
    if let Err(err) = services::messages::save(&state.pool, msg).await {
        log::error!("handle message error: {}", err);
    }
    Ok(())
}
