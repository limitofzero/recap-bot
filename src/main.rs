use std::time::Instant;

use sqlx::{migrate, postgres::PgPoolOptions, PgPool};
use teloxide::dispatching::dialogue::GetChatId;
use teloxide::prelude::*;
use teloxide::types::MessageKind;

use crate::errors::AppError;
pub mod domain;
pub mod errors;
pub mod health;

#[derive(Debug)]
pub struct InsertChat<'a> {
    chat_id: i64,
    title: &'a str,
}

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

    let msg_handler = |_: Bot, msg: Message, pool: PgPool| async move {
        let _ = save_message(&pool, msg)
            .await
            .inspect_err(|err| {
                log::error!("handle message error: {}", err);
            })
            .inspect_err(|err| {
                log::error!("msg handler error: {}", err);
            });

        Ok::<(), teloxide::RequestError>(())
    };

    let common_handler = dptree::entry()
        .branch(Update::filter_message().endpoint(msg_handler))
        .branch(Update::filter_edited_message().endpoint(msg_handler));

    log::info!("start metrics recorder");
    let metrics_handler = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .expect("metrics recorder installation is failed");
    log::info!("metrics recorder installed");

    let pool_for_health = pool.clone();
    tokio::spawn(async move {
        if let Err(err) = health::run(pool_for_health, metrics_handler, 8080).await {
            log::error!("health server is crushed: {}", err);
        }
    });

    let mut dispatcher = Dispatcher::builder(bot, common_handler)
        .dependencies(dptree::deps![pool])
        .enable_ctrlc_handler()
        .build();

    log::info!("Dispatcher starting");
    dispatcher.dispatch().await;
}

async fn insert_chat(pool: &PgPool, chat: InsertChat<'_>) -> Result<(), AppError> {
    let chat_id = chat.chat_id;
    let title = chat.title;

    let db_query_start = Instant::now();
    sqlx::query!(
        r#"
                INSERT INTO chats (id, title)
                VALUES ($1, $2)
                ON CONFLICT (id) DO UPDATE SET title = EXCLUDED.title
                WHERE chats.title IS DISTINCT FROM EXCLUDED.title
            "#,
        chat_id,
        title,
    )
    .execute(pool)
    .await
    .map_err(|err| AppError::InsertChatError(chat_id, err.to_string()))?;

    metrics::histogram!("bot_db_query_seconds", "operation" => "insert_chat")
        .record(db_query_start.elapsed().as_secs_f64());
    Ok(())
}

async fn save_message(pool: &PgPool, msg: Message) -> Result<(), AppError> {
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
        insert_chat(
            pool,
            InsertChat {
                chat_id,
                title: chat_title,
            },
        )
        .await?;

        let text = msg.text().unwrap_or_default();
        let edited_at = msg.edit_date().copied();

        let db_query_start = Instant::now();
        sqlx::query!(
            r#"
                        INSERT INTO messages (
                        chat_id,
                        message_id,
                        user_id,
                        text,
                        created_at,
                        edited_at
                        )
                        VALUES ($1, $2, $3, $4, $5, $6)
                        ON CONFLICT (chat_id, message_id)
                        DO UPDATE
                        SET text = EXCLUDED.text,
                        edited_at = COALESCE(EXCLUDED.edited_at, messages.edited_at)
                    "#,
            chat_id,
            message_id,
            user_id as i64,
            text,
            created_at,
            edited_at
        )
        .execute(pool)
        .await
        .inspect(|_| {
            metrics::histogram!("bot_db_query_seconds", "operation" => "insert_message")
                .record(db_query_start.elapsed().as_secs_f64());
            log::debug!("message was saved (chat={}, msg={})", chat_id, message_id);
        })
        .map_err(|err| AppError::SaveMessage(err.to_string()))?;
    }

    Ok(())
}
