use redis::aio::ConnectionManager;
use redis::AsyncTypedCommands;
use std::time::Duration;

#[derive(Clone)]
pub struct RateLimiter {
    connection: Option<ConnectionManager>,
    limit: isize,
    window_secs: i64,
}

pub struct RateLimited;

const KEY: &str = "ratelimit:user:";

impl RateLimiter {
    pub fn new(
        connection: Option<ConnectionManager>,
        limit: usize,
        window_inderval: Duration,
    ) -> Self {
        Self {
            connection,
            limit: limit as isize,
            window_secs: window_inderval.as_secs() as i64,
        }
    }

    pub async fn check(&self, user_id: u64) -> Result<(), RateLimited> {
        let Some(mut connection) = self.connection.clone() else {
            log::warn!("redis connection is not set");
            return Ok(());
        };

        let key = format!("{KEY}{user_id}");

        let count = match connection.incr(&key, 1).await {
            Ok(c) => c,
            Err(err) => {
                log::error!("redis incr failed: {}", err);
                return Ok(());
            }
        };

        if count == 1 {
            let _ = connection.expire(key, self.window_secs).await;
        }

        if count > self.limit {
            return Err(RateLimited);
        }

        Ok(())
    }
}
