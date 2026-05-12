use std::sync::Arc;

use sqlx::PgPool;

use crate::services::ai_client::AiClient;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub ai_client: Arc<AiClient>,
    pub ai_recap_system_prompt: String,
}
