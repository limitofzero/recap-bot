use std::time::Duration;

use crate::config::Config;
use crate::domain::commands::Command;
use crate::infra::ai_client::AiClient;
use crate::infra::rate_limiter::RateLimiter;
use redis::aio::ConnectionManager;
use sqlx::{migrate, postgres::PgPoolOptions};
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

mod app;
mod commands;
mod config;
mod domain;
mod errors;
mod formatters;
mod handlers;
mod health;
mod infra;
mod metrics;
mod repositories;
mod services;
mod shutdown;
mod validators;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let cfg = Config::from_env();
    log::info!("config loaded: {:?}", cfg);

    let bot = Bot::from_env();
    if let Err(err) = bot.set_my_commands(Command::bot_commands()).await {
        log::warn!("failed to register bot commands: {}", err);
    }

    let pool = PgPoolOptions::new()
        .max_connections(3)
        .connect(&cfg.database_url)
        .await
        .inspect(|_| log::info!("DB is connected"))
        .expect("Failed to connect db");

    migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations failed");
    log::info!("migrations are ok");

    log::info!("start metrics recorder");
    let metrics_handler = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .expect("metrics recorder installation is failed");
    log::info!("metrics recorder installed");

    let ai_client = AiClient::new(cfg.ai_api_key, cfg.ai_api_url, cfg.ai_model);
    let shutdown_token = shutdown::get_shutdown_token();

    let redis_client = redis::Client::open(cfg.redis_url).expect("invalid redis url");
    let redis_connection = ConnectionManager::new(redis_client)
        .await
        .inspect_err(|err| log::warn!("redis connection is failed: {:?}", err))
        .ok();

    let rate_limiter = RateLimiter::new(
        redis_connection,
        cfg.rate_per_user,
        Duration::from_secs(3600),
    );

    let me = bot.get_me().await.expect("get_me failed");
    let bot_id = me.id.0;
    let bot_name = me.user.username.expect("bot name is undefined");

    let state = app::AppState::new(pool.clone(), ai_client, rate_limiter, bot_name, bot_id);

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
