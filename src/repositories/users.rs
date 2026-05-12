use sqlx::PgPool;
use std::time::Instant;

use crate::errors::AppError;

pub async fn upsert_user(
    pool: &PgPool,
    id: i64,
    first_name: &str,
    last_name: Option<&str>,
    is_bot: bool,
    username: Option<&str>,
    is_premium: bool,
) -> Result<(), AppError> {
    let started = Instant::now();

    sqlx::query!(
        r#"
            INSERT INTO users (
                id,
                first_name,
                last_name,
                is_bot,
                username,
                is_premium,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (id)
            DO UPDATE
            SET
                first_name = EXCLUDED.first_name,
                last_name = EXCLUDED.last_name,
                is_bot = EXCLUDED.is_bot,
                username = EXCLUDED.username,
                is_premium = EXCLUDED.is_premium,
                updated_at = NOW()
            WHERE
                users.first_name IS DISTINCT FROM EXCLUDED.first_name
                OR users.last_name  IS DISTINCT FROM EXCLUDED.last_name
                OR users.is_bot     IS DISTINCT FROM EXCLUDED.is_bot
                OR users.username   IS DISTINCT FROM EXCLUDED.username
                OR users.is_premium IS DISTINCT FROM EXCLUDED.is_premium
        "#,
        id,
        first_name,
        last_name,
        is_bot,
        username,
        is_premium
    )
    .execute(pool)
    .await
    .map_err(|err| {
        AppError::DbError(format!(
            "insert user {}, first_name: {}, err: {}",
            id, first_name, err
        ))
    })?;

    metrics::histogram!("bot_db_query_seconds", "operation" => "insert_user")
        .record(started.elapsed().as_secs_f64());

    Ok(())
}
