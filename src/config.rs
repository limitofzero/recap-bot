use crate::domain::consts::DEFAULT_RATE_LIMIT_PER_USER;

#[derive(serde::Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,

    pub ai_api_key: String,
    pub ai_api_url: String,
    pub ai_model: String,

    #[serde(default = "default_rate_per_user")]
    pub rate_per_user: usize,
}

fn default_rate_per_user() -> usize {
    DEFAULT_RATE_LIMIT_PER_USER
}

impl Config {
    pub fn from_env() -> Self {
        envy::from_env::<Self>().unwrap_or_else(|e| panic!("failed to load config from env: {e}"))
    }
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("database_url", &"***")
            .field("redis_url", &"***")
            .field("ai_api_key", &"***")
            .field("ai_api_url", &self.ai_api_url)
            .field("ai_model", &self.ai_model)
            .field("rate_per_user", &self.rate_per_user)
            .finish()
    }
}
