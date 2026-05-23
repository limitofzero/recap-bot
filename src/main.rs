use std::time::Duration;

use crate::domain::commands::Command;
use crate::domain::consts::DEFAULT_RATE_LIMIT_PER_USER;
use crate::infra::ai_client::AiClient;
use crate::infra::rate_limiter::RateLimiter;
use redis::aio::ConnectionManager;
use sqlx::{migrate, postgres::PgPoolOptions};
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

mod app;
mod commands;
mod domain;
mod errors;
mod handlers;
mod health;
mod infra;
mod metrics;
mod repositories;
mod services;
mod shutdown;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let bot = Bot::from_env();
    if let Err(err) = bot.set_my_commands(Command::bot_commands()).await {
        log::warn!("failed to register bot commands: {}", err);
    }

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(3)
        .connect(&db_url)
        .await
        .inspect(|_| log::info!("DB is connected"))
        .expect("Failed to connect db");

    migrate!("./migrations")
        .run(&pool)
        .await
        .inspect(|_| log::info!("migrations are ok"))
        .expect("migrations failed");

    log::info!("start metrics recorder");
    let metrics_handler = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .expect("metrics recorder installation is failed");
    log::info!("metrics recorder installed");

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

    let ai_api_key = std::env::var("AI_API_KEY").expect("AI_API_KEY must be set");
    let ai_url = std::env::var("AI_API_URL").expect("AI_API_URL must be set");
    let ai_model = std::env::var("AI_MODEL").expect("AI_MODEL must be set");
    let ai_client = AiClient::new(ai_api_key, ai_url, ai_model);
    let shutdown_token = shutdown::get_shutdown_token();

    let redis_client = redis::Client::open(redis_url).expect("invalid redis url");
    let redis_connection = ConnectionManager::new(redis_client)
        .await
        .inspect_err(|err| log::warn!("redis connection is failed: {:?}", err))
        .ok();

    let rate_per_user: usize = std::env::var("RATE_PER_USER")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_RATE_LIMIT_PER_USER);

    let rate_limiter = RateLimiter::new(redis_connection, rate_per_user, Duration::from_hours(1));

    let state = app::AppState::new(pool.clone(), ai_client, rate_limiter);

    let pool_for_health = pool.clone();
    let health_shutdown_token = shutdown_token.clone();
    tokio::spawn(async move {
        if let Err(err) = health::run(
            pool_for_health,
            metrics_handler,
            8080,
            health_shutdown_token,
        )
        .await
        {
            log::error!("health server is crashed: {}", err);
        }
    });

    let handler = dptree::entry()
        .branch(handlers::commands::router())
        .branch(Update::filter_message().endpoint(handlers::message::handle))
        .branch(Update::filter_edited_message().endpoint(handlers::message::handle));

    let mut dispatcher = Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state])
        .build();

    let dispatcher_shutdown = dispatcher.shutdown_token();
    let dispatch = tokio::spawn(async move {
        log::info!("Dispatcher starting");
        dispatcher.dispatch().await
    });

    shutdown_token.cancelled().await;
    let _ = dispatcher_shutdown.shutdown();
    log::info!("shutdown signal received, draining...");

    match tokio::time::timeout(Duration::from_secs(25), dispatch).await {
        Ok(_) => log::info!("drained cleanly"),
        Err(_) => log::warn!("drain timeout"),
    };

    pool.close().await;
}
