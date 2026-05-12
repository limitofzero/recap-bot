use std::time::Instant;

use sqlx::PgPool;

use crate::errors::AppError;

pub async fn upsert(pool: &PgPool, chat_id: i64, title: &str) -> Result<(), AppError> {
    let started = Instant::now();

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
        .record(started.elapsed().as_secs_f64());

    Ok(())
}
