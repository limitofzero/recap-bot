use std::sync::Arc;
use std::time::Duration;

use crate::domain::commands::Command;
use crate::services::ai_client::AiClient;
use sqlx::{migrate, postgres::PgPoolOptions};
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;

mod app;
mod commands;
mod domain;
mod errors;
mod handlers;
mod health;
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

    let ai_api_key = std::env::var("AI_API_KEY").expect("AI_API_KEY must be set");
    let ai_url = std::env::var("AI_API_URL").expect("AI_API_URL must be set");
    let ai_system_propmt = std::env::var("AI_SYSTEM_PROMPT").expect("AI_SYSTEM_PROMPT must be set");
    let ai_model = std::env::var("AI_MODEL").expect("AI_MODEL must be set");
    let ai_client = AiClient::new(ai_api_key, ai_url, ai_model);
    let shutdown_token = shutdown::get_shutdown_token();

    let state = app::AppState {
        pool: pool.clone(),
        ai_client: Arc::new(ai_client),
        ai_recap_system_prompt: ai_system_propmt,
    };

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
