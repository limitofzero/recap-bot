use std::{sync::Arc, time::Duration};

use reqwest::Client;

use crate::errors::AppError;

#[derive(Debug, serde::Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, serde::Serialize)]
struct RequestBody {
    messages: Vec<Message>,
    model: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct ChatResponse {
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Choice {
    pub message: ResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ResponseMessage {
    pub content: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

pub struct AiClient {
    client: Arc<Client>,
    api_key: String,
    uri: String,
    model: String,
}

impl AiClient {
    pub fn new(api_key: String, uri: String, model: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .expect("failed to build reqwest client");
        Self {
            client: Arc::new(client),
            api_key,
            uri,
            model,
        }
    }

    pub async fn make_request(&self, system_prompt: &str, user_prompt: String) -> Result<String, AppError> {
        let body = RequestBody {
            model: self.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: system_prompt.to_string()
                },
                Message {
                    role: "user".to_string(),
                    content: user_prompt,
                }
            ]
        };

        let uri = self.uri.clone();
        let api_key = self.api_key.clone();
        let result = self.client.post(uri)
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await
            .map_err(|err| AppError::AiResponse(err.to_string()))?;

        if !result.status().is_success() {
            let status = result.status();
            let body = result.text().await.unwrap_or_default();
            return Err(AppError::AiResponse(format!("status: {}, body: {}", status, body)));
        }

        let parsed: ChatResponse = result.json().await
            .map_err(|e| AppError::AiResponse(format!("parse: {}", e)))?;

        let choices = parsed.choices
            .into_iter()
            .next()
            .ok_or_else(|| AppError::AiResponse("no choinces in response".to_string()))?;

        Ok(choices.message.content)
    }
}