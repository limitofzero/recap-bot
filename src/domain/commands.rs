use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone, Debug)]
#[command(rename_rule = "snake_case", description = "Available commands:")]
pub enum Command {
    #[command(description = "Show help")]
    Help,

    #[command(description = "Summarize last N messages. Usage: /recap 100, max: 100")]
    Recap(usize),
}
