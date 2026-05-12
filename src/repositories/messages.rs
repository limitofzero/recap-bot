use std::time::Instant;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::errors::AppError;

#[derive(Debug, serde::Serialize)]
pub struct StoredMessage {
    pub user_id: Option<i64>,
    pub text: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub async fn upsert(
    pool: &PgPool,
    chat_id: i64,
    message_id: i64,
    user_id: i64,
    text: &str,
    created_at: DateTime<Utc>,
    edited_at: Option<DateTime<Utc>>,
) -> Result<(), AppError> {
    let started = Instant::now();

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
        user_id,
        text,
        created_at,
        edited_at,
    )
    .execute(pool)
    .await
    .map_err(|err| AppError::SaveMessage(err.to_string()))?;

    metrics::histogram!("bot_db_query_seconds", "operation" => "insert_message")
        .record(started.elapsed().as_secs_f64());

    Ok(())
}


pub async fn get_last(
    pool: &PgPool,
    chat_id: i64,
    count: i64,
) -> Result<Vec<StoredMessage>, AppError> {
    let started = Instant::now();

    let messages= sqlx::query_as!(
        StoredMessage,
        r#"
            SELECT user_id, text, created_at
            FROM messages
            WHERE chat_id = $1
            ORDER BY created_at DESC
            LIMIT $2
        "#,
        chat_id,
        count
    )
    .fetch_all(pool)
    .await
    .map_err(|err| AppError::SelectMessagesError(chat_id, err.to_string()))?;

    metrics::histogram!("bot_db_query_seconds", "operation" => "select_message")
        .record(started.elapsed().as_secs_f64());

    Ok(messages)
}