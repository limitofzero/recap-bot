use sqlx::{migrate, postgres::PgPoolOptions};
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let bot = Bot::from_env();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(3)
        .connect(&db_url)
        .await
        .inspect(|_| {
            log::info!("DB is connected");
        })
        .expect("Failed to connect db");

    migrate!("./migrations")
        .run(&pool)
        .await
        .inspect(|_| {
            log::info!("migrations are ok");
        })
        .expect("migrations failed");

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        bot.send_dice(msg.chat.id).await?;
        Ok(())
    })
    .await;
}
