use std::time::Instant;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::errors::AppError;

#[derive(Debug, serde::Serialize)]
pub struct StoredMessage {
    pub user_id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub text: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub async fn upsert(
    pool: &PgPool,
    chat_id: i64,
    message_id: i64,
    user_id: Option<i64>,
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

    let messages = sqlx::query_as!(
        StoredMessage,
        r#"
            SELECT
                m.user_id           AS "user_id!: i64",
                u.username          AS "username?: String",
                u.first_name        AS "first_name!: String",
                m.text              AS "text?: String",
                m.created_at        AS "created_at!: DateTime<Utc>"
            FROM messages m
            INNER JOIN users u ON u.id = m.user_id
            WHERE m.chat_id = $1
            ORDER BY m.created_at DESC
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
