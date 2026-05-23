use std::{collections::HashMap, sync::Arc};

use sqlx::PgPool;

use crate::{
    domain::promts::Prompt,
    infra::{ai_client::AiClient, rate_limiter::RateLimiter},
};

const SYSTEM_RECAP_PROMPT: &str = include_str!("../prompts/system.txt");
const SYSTEM_TOP_MEMBERS_PROMPT: &str = include_str!("../prompts/top_members.txt");

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub ai_client: Arc<AiClient>,
    pub ai_system_propmts: HashMap<Prompt, &'static str>,
    pub rate_limiter: RateLimiter,
}

impl AppState {
    pub fn new(pool: PgPool, ai_client: AiClient, rate_limiter: RateLimiter) -> Self {
        Self {
            pool,
            rate_limiter,
            ai_client: Arc::new(ai_client),
            ai_system_propmts: Self::init_prompts(),
        }
    }

    fn init_prompts() -> HashMap<Prompt, &'static str> {
        let mut promts = HashMap::new();

        promts.insert(Prompt::Recap, SYSTEM_RECAP_PROMPT);
        promts.insert(Prompt::TopMembers, SYSTEM_TOP_MEMBERS_PROMPT);

        promts
    }
}
