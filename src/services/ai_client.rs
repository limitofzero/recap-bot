use std::time::Duration;

use reqwest::Client;

use crate::errors::AppError;

#[derive(Debug, serde::Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, serde::Serialize)]
struct RequestBody<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
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
    client: Client,
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
            client,
            api_key,
            uri,
            model,
        }
    }

    pub async fn make_request(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, AppError> {
        let body = RequestBody {
            model: &self.model,
            messages: vec![
                Message { role: "system", content: &system_prompt },
                Message { role: "user",   content: user_prompt },
            ],
        };

        let response = self
            .client
            .post(&self.uri)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|err| AppError::AiResponse(format!("send: {}", err)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::AiResponse(format!(
                "status: {}, body: {}",
                status, body
            )));
        }

        let parsed: ChatResponse = response
            .json()
            .await
            .map_err(|e| AppError::AiResponse(format!("parse: {}", e)))?;

        if let Some(usage) = &parsed.usage {
            log::debug!(
                "LLM usage: prompt={} completion={} total={}",
                usage.prompt_tokens,
                usage.completion_tokens,
                usage.total_tokens
            );
            metrics::counter!("bot_llm_tokens_total", "kind" => "prompt")
                .increment(usage.prompt_tokens as u64);
            metrics::counter!("bot_llm_tokens_total", "kind" => "completion")
                .increment(usage.completion_tokens as u64);
        }

        let choice = parsed
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| AppError::AiResponse("no choices in response".to_string()))?;

        if choice.finish_reason.as_deref() == Some("length") {
            log::warn!("LLM response was truncated due to length limit");
        }

        Ok(choice.message.content)
    }
}
