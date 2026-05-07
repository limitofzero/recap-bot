use sqlx::{migrate, postgres::PgPoolOptions, PgPool};
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::prelude::*;
use teloxide::types::MessageKind;

use crate::errors::AppError;
pub mod domain;
pub mod errors;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let bot = Bot::from_env();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(3)
        .connect(&db_url)
        .await
        .inspect(|_| {
            log::info!("DB is connected");
        })
        .expect("Failed to connect db");

    migrate!("./migrations")
        .run(&pool)
        .await
        .inspect(|_| {
            log::info!("migrations are ok");
        })
        .expect("migrations failed");

    let _ = Dispatcher::builder(bot.clone(), Update::filter_message().endpoint(handle_message));

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        bot.send_dice(msg.chat.id).await?;
        Ok(())
    })
    .await;
}

async fn handle_message(_: Bot, msg: Message, pool: &PgPool) -> ResponseResult<()> {
    let _ = save_message(pool, msg).await.inspect_err(|err| {
        log::error!("handle message error: {}", err.to_string());
    });

    Ok(())
}

async fn save_message(pool: &PgPool, msg: Message) -> Result<(), AppError> {
    let message_id = msg.id.0 as i64;
    let chat_id = msg
        .chat_id()
        .map(|id| id.0)
        .ok_or(AppError::EmptyChatId(message_id))?;
    let created_at = msg.date;
    let user_id = msg
        .from
        .as_ref()
        .map(|u| u.id.0)
        .ok_or(AppError::EmptyUserId(message_id, chat_id))?;

    match &msg.kind {
        MessageKind::Common(content) => {
            let text = msg.text().unwrap_or_default();
            if let Some(edited_date) = content.edit_date {
                sqlx::query!(
                    r#"
                        UPDATE messages
                        SET text = $1,
                        edited_at = $2,
                        chat_id = $3,
                        message_id = $4
                        WHERE chat_id = $3 AND message_id = $4;
                    "#,
                    text,
                    edited_date,
                    chat_id,
                    message_id,
                ).execute(pool)
                .await
                .map_err(|err| AppError::UpdateMessage(err.to_string()))?;
            } else {
                sqlx::query!(
                    r#"
                        INSERT INTO messages (
                        chat_id,
                        message_id,
                        user_id,
                        text,
                        created_at
                        )
                        VALUES ($1, $2, $3, $4, $5)
                        ON CONFLICT (chat_id, message_id) DO NOTHING
                    "#,
                    chat_id,
                    message_id,
                    user_id as i64,
                    text,
                    created_at,
                )
                .execute(pool)
                .await
                .map_err(|err| AppError::SaveMessage(err.to_string()))?;
            }
        }
        _ => {}
    };

    Ok(())
}
