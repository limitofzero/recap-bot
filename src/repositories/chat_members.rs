use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::time::Instant;

use crate::errors::AppError;

#[derive(Debug, serde::Serialize)]
pub struct MemberWithMessages {
    pub id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub message_count: i64,
    pub last_seen_at: DateTime<Utc>,
    pub is_premium: bool,
}

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

    metrics::histogram!("bot_db_query_seconds", "operation" => "insert_chat_member")
        .record(started.elapsed().as_secs_f64());

    Ok(())
}

pub async fn get_top_members(
    pool: &PgPool,
    chat_id: i64,
    count: i64,
) -> Result<Vec<MemberWithMessages>, AppError> {
    let started = Instant::now();

    let top_members = sqlx::query_as!(
        MemberWithMessages,
        r#"
            SELECT
                u.id AS "id!: i64",
                u.username AS "username?: String",
                u.first_name AS "first_name!: String",
                u.is_premium AS "is_premium!: bool",
                ch.last_seen_at AS "last_seen_at!: DateTime<Utc>",
                ch.message_count AS "message_count!: i64"
            FROM chat_members ch
            INNER JOIN users u ON ch.user_id = u.id
            WHERE ch.chat_id = $1 and u.is_bot IS NOT TRUE
            ORDER BY ch.message_count DESC
            LIMIT $2
        "#,
        chat_id,
        count
    )
    .fetch_all(pool)
    .await
    .map_err(|err| AppError::DbError(format!("get top members for {}, err: {}", chat_id, err)))?;

    metrics::histogram!("bot_db_query_seconds", "operation" => "select_top_members")
        .record(started.elapsed().as_secs_f64());

    Ok(top_members)
}
