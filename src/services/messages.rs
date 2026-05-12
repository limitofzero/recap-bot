use sqlx::PgPool;
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::types::{Message, MessageKind};

use crate::errors::AppError;
use crate::repositories;

pub async fn save(pool: &PgPool, msg: Message) -> Result<(), AppError> {
    metrics::counter!("bot_messages_received_total").increment(1);

    let message_id = msg.id.0 as i64;
    let chat_id = msg
        .chat_id()
        .map(|id| id.0)
        .ok_or(AppError::EmptyChatId(message_id))?;
    let chat_title = msg.chat.title().unwrap_or_default();
    let created_at = msg.date;
    let user_id = msg
        .from
        .as_ref()
        .map(|u| u.id.0)
        .ok_or(AppError::EmptyUserId(message_id, chat_id))?;

    if let MessageKind::Common(_) = &msg.kind {
        repositories::chats::upsert(pool, chat_id, chat_title).await?;

        if let Some(user) = msg.from.as_ref() {
            repositories::users::upsert_user(
                pool,
                user.id.0 as i64,
                &user.first_name,
                user.last_name.as_deref(),
                user.is_bot,
                user.username.as_deref(),
                user.is_premium
            ).await?;

            repositories::chat_members::upsert_chat_member(
                pool,
                chat_id,
                user.id.0 as i64,
                true,
                None,
            ).await?;
        }

        let text = msg.text().unwrap_or_default();
        let edited_at = msg.edit_date().copied();

        repositories::messages::upsert(
            pool,
            chat_id,
            message_id,
            user_id as i64,
            text,
            created_at,
            edited_at,
        )
        .await?;

        log::debug!("message was saved (chat={}, msg={})", chat_id, message_id);
    }

    Ok(())
}
