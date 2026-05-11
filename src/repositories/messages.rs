use std::time::Instant;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::errors::AppError;

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
