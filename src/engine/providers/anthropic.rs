//! Anthropic Claude Translation Provider.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub struct AnthropicProvider {
    pub api_key: String,
    pub model: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: Option<String>, model: Option<String>) -> Self {
        let key = api_key
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
            .unwrap_or_default();
        let mdl = model
            .or_else(|| std::env::var("ANTHROPIC_MODEL").ok())
            .unwrap_or_else(|| "claude-3-5-sonnet-20241022".to_string());

        Self {
            api_key: key,
            model: mdl,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct MessagePayload {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<MessagePayload>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct JsonOutput {
    translations: Vec<String>,
}

#[async_trait]
impl super::super::TranslationProvider for AnthropicProvider {
    async fn translate_batch(
        &self,
        items: &[String],
        target_locale: &str,
    ) -> Result<Vec<String>, super::super::EngineError> {
        let endpoint = "https://api.anthropic.com/v1/messages";

        let system_prompt = format!(
            "You are a software localization system. Translate the given array of strings into '{target_locale}'.\n\
            Preserve all placeholders like {{var}} and numbered markup tags like <0>...</0> exactly.\n\
            Return strictly a JSON object: {{ \"translations\": [\"...\"] }}."
        );

        let input_json = serde_json::to_string(items).unwrap_or_default();

        let req_body = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 4096,
            system: system_prompt,
            messages: vec![MessagePayload {
                role: "user".to_string(),
                content: input_json,
            }],
        };

        let resp = self
            .client
            .post(endpoint)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&req_body)
            .send()
            .await
            .map_err(|e| super::super::EngineError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(super::super::EngineError::Api(format!(
                "Anthropic returned HTTP {}",
                resp.status()
            )));
        }

        let anthropic_res: AnthropicResponse = resp
            .json()
            .await
            .map_err(|e| super::super::EngineError::Json(e.to_string()))?;

        let first = anthropic_res
            .content
            .first()
            .ok_or_else(|| super::super::EngineError::Api("Empty response from Claude".to_string()))?;

        let parsed: JsonOutput = serde_json::from_str(&first.text)
            .map_err(|e| super::super::EngineError::Json(format!("Failed to parse Claude JSON: {e}")))?;

        Ok(parsed.translations)
    }

    fn provider_name(&self) -> &'static str {
        "anthropic"
    }
}
