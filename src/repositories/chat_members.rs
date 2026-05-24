use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::time::Instant;

use crate::errors::AppError;
use crate::metrics::{self, DbOp};

#[derive(Debug)]
struct FlatMemberWithMessage {
    pub id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub message_count: i64,
    pub last_seen_at: DateTime<Utc>,
    pub is_premium: bool,
    pub message_text: String,
    pub message_created_at: DateTime<Utc>,
}

#[derive(Debug, serde::Serialize)]
pub struct Message {
    pub text: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, serde::Serialize)]
pub struct MemberWithMessages {
    pub id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub message_count: i64,
    pub last_seen_at: DateTime<Utc>,
    pub is_premium: bool,
    pub messages: Vec<Message>,
}

pub async fn get_top_members_with_messages(
    pool: &PgPool,
    chat_id: i64,
    top_users_count: i64,
    messages_count: i64,
) -> Result<Vec<MemberWithMessages>, AppError> {
    let started = Instant::now();

    let result = sqlx::query_as!(
        FlatMemberWithMessage,
        r#"
            WITH top_users AS (
                SELECT
                    u.id,
                    u.first_name,
                    u.username,
                    u.is_premium,
                    ch.message_count,
                    ch.last_seen_at
                from chat_members ch
                INNER JOIN users u ON u.id = ch.user_id
                WHERE ch.chat_id = $1 AND u.is_bot IS NOT TRUE
                ORDER BY ch.message_count DESC
                LIMIT $2
            )
            SELECT
                tu.id AS "id!: i64",
                tu.first_name AS "first_name!: String",
                tu.username AS "username?: String",
                tu.is_premium AS "is_premium!: bool",
                tu.message_count AS "message_count!: i64",
                tu.last_seen_at AS "last_seen_at!: DateTime<Utc>",
                m.text AS "message_text!: String",
                m.created_at AS "message_created_at!: DateTime<Utc>"
            FROM top_users tu
            INNER JOIN LATERAL (
                SELECT
                    text,
                    created_at
                FROM messages
                WHERE user_id = tu.id AND chat_id = $1 AND text IS NOT NULL AND text <> ''
                ORDER BY created_at DESC
                LIMIT $3
            ) m ON TRUE
            ORDER BY tu.message_count DESC, m.created_at DESC
        "#,
        chat_id,
        top_users_count,
        messages_count,
    )
    .fetch_all(pool)
    .await
    .map_err(|err| {
        AppError::DbError(format!(
            "select top members with messages with chat_id {} error: {}",
            chat_id, err
        ))
    })?;

    metrics::db_query(DbOp::SelectTopMembers, started.elapsed());

    Ok(flat_flat_top_users(result))
}

fn flat_flat_top_users(rows: Vec<FlatMemberWithMessage>) -> Vec<MemberWithMessages> {
    let mut result = HashMap::new();

    for row in rows {
        let entry = result.entry(row.id).or_insert_with(|| MemberWithMessages {
            id: row.id,
            message_count: row.message_count,
            is_premium: row.is_premium,
            username: row.username,
            first_name: row.first_name,
            last_seen_at: row.last_seen_at,
            messages: Vec::with_capacity(1),
        });

        entry.messages.push(Message {
            created_at: row.message_created_at,
            text: row.message_text,
        });
    }

    let mut as_vec: Vec<_> = result.into_values().collect();
    as_vec.sort_by_key(|a| Reverse(a.message_count));
    as_vec
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

    metrics::db_query(DbOp::UpsertChatMember, started.elapsed());

    Ok(())
}
