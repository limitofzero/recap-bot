use std::sync::Arc;

use sqlx::{migrate, postgres::PgPoolOptions};
use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use crate::domain::commands::Command;
use crate::services::ai_client::{self, AiClient};

mod app;
mod domain;
mod errors;
mod handlers;
mod health;
mod repositories;
mod services;
mod commands;



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
    let ai_model = std::env::var("AI_MODEL").unwrap_or("glm-4".to_string());
    let ai_client = AiClient::new(ai_api_key, ai_url, ai_model);

    let state = app::AppState { pool: pool.clone(), ai_client: Arc::new(ai_client) };

    let pool_for_health = pool.clone();
    tokio::spawn(async move {
        if let Err(err) = health::run(pool_for_health, metrics_handler, 8080).await {
            log::error!("health server is crushed: {}", err);
        }
    });

    let handler = dptree::entry()
        .branch(handlers::commands::router())
        .branch(Update::filter_message().endpoint(handlers::message::handle))
        .branch(Update::filter_edited_message().endpoint(handlers::message::handle));

    let mut dispatcher = Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![state])
        .enable_ctrlc_handler()
        .build();

    log::info!("Dispatcher starting");
    dispatcher.dispatch().await;
}
