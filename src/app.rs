use std::{collections::HashMap, sync::Arc};

use sqlx::PgPool;

use crate::{
    domain::promts::Prompt,
    errors::AppError,
    infra::{ai_client::AiClient, rate_limiter::RateLimiter},
};

const SYSTEM_RECAP_PROMPT: &str = include_str!("../prompts/system.txt");
const SYSTEM_TOP_MEMBERS_PROMPT: &str = include_str!("../prompts/top_members.txt");
const RESPONSE_TO_USER: &str = include_str!("../prompts/response_to_user.txt");
const USER_RECAP_PROMPT: &str = include_str!("../prompts/user_recap.txt");

#[derive(Clone)]
pub struct Bot {
    pub name: String,
    pub id: u64,
}

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub ai_client: Arc<AiClient>,
    pub rate_limiter: RateLimiter,
    pub bot: Bot,
    ai_system_propmts: HashMap<Prompt, &'static str>,
}

impl AppState {
    pub fn new(
        pool: PgPool,
        ai_client: AiClient,
        rate_limiter: RateLimiter,
        bot_name: String,
        bot_id: u64,
    ) -> Self {
        Self {
            pool,
            rate_limiter,
            bot: Bot {
                id: bot_id,
                name: bot_name,
            },
            ai_client: Arc::new(ai_client),
            ai_system_propmts: Self::init_prompts(),
        }
    }

    pub fn get_promt_or_error(&self, prompt: Prompt) -> Result<&str, AppError> {
        match self.ai_system_propmts.get(&prompt) {
            Some(prompt) => Ok(prompt),
            None => Err(AppError::PromptNotFound(prompt)),
        }
    }

    fn init_prompts() -> HashMap<Prompt, &'static str> {
        let mut promts = HashMap::new();

        promts.insert(Prompt::Recap, SYSTEM_RECAP_PROMPT);
        promts.insert(Prompt::TopMembers, SYSTEM_TOP_MEMBERS_PROMPT);
        promts.insert(Prompt::ResponseToUser, RESPONSE_TO_USER);
        promts.insert(Prompt::UserRecap, USER_RECAP_PROMPT);

        promts
    }
}
