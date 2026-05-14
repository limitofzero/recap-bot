use std::time::Instant;

use sqlx::PgPool;

use crate::errors::AppError;
use crate::metrics::{self, DbOp};

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

    metrics::db_query(DbOp::InsertChat, started.elapsed());

    Ok(())
}
