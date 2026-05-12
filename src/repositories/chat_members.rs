use sqlx::PgPool;
use std::time::Instant;

use crate::errors::AppError;

pub async fn upsert_chat_member(
    pool: &PgPool,
    chat_id: i64,
    user_id: i64,
    is_message: bool,
    nickname_in_chat: Option<&str>,
) -> Result<(), AppError> {
    let started = Instant::now();

    let message_count: i64 = if is_message { 1 } else { 0 };

    sqlx::query!(
        r#"
            INSERT INTO chat_members (
                chat_id,
                user_id,
                message_count,
                nickname_in_chat,
                joined_at,
                last_seen_at
            )
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            ON CONFLICT (chat_id, user_id)
            DO UPDATE
            SET
                message_count = chat_members.message_count + 1,
                nickname_in_chat = EXCLUDED.nickname_in_chat,
                last_seen_at = NOW(),
                deleted_at = NULL
        "#,
        chat_id,
        user_id,
        message_count,
        nickname_in_chat
    )
    .execute(pool)
    .await
    .map_err(|err| {
        AppError::DbError(format!(
            "touch chat_member, chat_id: {}, user_id: {}, err: {}",
            chat_id, user_id, err
        ))
    })?;

    metrics::histogram!("bot_db_query_seconds", "operation" => "insert_user")
        .record(started.elapsed().as_secs_f64());

    Ok(())
}
