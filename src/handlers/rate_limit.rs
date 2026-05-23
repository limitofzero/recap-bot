use teloxide::{
    payloads::SendMessageSetters,
    prelude::{Bot, Requester},
    types::{Message, ReplyParameters},
    RequestError,
};

use crate::infra::rate_limiter::RateLimiter;

const REJECT_MESSAGE: &str = "Твой лимит превышен, так что иди нахуй... отдохни часок";

pub async fn allowed(
    bot: &Bot,
    msg: &Message,
    limiter: &RateLimiter,
) -> Result<bool, RequestError> {
    let Some(user_id) = msg.from.as_ref().map(|u| u.id.0) else {
        return Ok(true);
    };

    if limiter.check(user_id).await.is_err() {
        bot.send_message(msg.chat.id, REJECT_MESSAGE)
            .reply_parameters(ReplyParameters::new(msg.id))
            .await?;
        return Ok(false);
    }

    Ok(true)
}
