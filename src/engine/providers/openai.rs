//! OpenAI-Compatible Translation Provider.
//!
//! Supports OpenAI, Groq, Together AI, DeepSeek, vLLM, and any compatible endpoint.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub struct OpenAiProvider {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
    client: reqwest::Client,
}

impl OpenAiProvider {
    pub fn new(base_url: Option<String>, api_key: Option<String>, model: Option<String>) -> Self {
        let url = base_url
            .or_else(|| std::env::var("OPENAI_BASE_URL").ok())
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
        let key = api_key
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .unwrap_or_default();
        let mdl = model
            .or_else(|| std::env::var("OPENAI_MODEL").ok())
            .unwrap_or_else(|| "gpt-4o-mini".to_string());

        Self {
            base_url: url,
            api_key: key,
            model: mdl,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    response_format: Option<ResponseFormat>,
}

#[derive(Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageContent,
}

#[derive(Deserialize)]
struct ChatMessageContent {
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct JsonTranslations {
    translations: Vec<String>,
}

#[async_trait]
impl super::super::TranslationProvider for OpenAiProvider {
    async fn translate_batch(
        &self,
        items: &[String],
        target_locale: &str,
    ) -> Result<Vec<String>, super::super::EngineError> {
        let endpoint = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let system_prompt = format!(
            "You are a professional software localization engine. Translate the following array of strings into locale '{target_locale}'.\n\
            STRICT RULES:\n\
            1. Preserve all placeholders like {{var}}, {{count}}, {{name}} EXACTLY as they are. Do NOT translate variable names.\n\
            2. Preserve all XML/HTML markup tags like <0>...</0>, <1>...</1> exactly with identical opening and closing tags.\n\
            3. Return output strictly as a JSON object with schema: {{ \"translations\": [\"...\"] }}."
        );

        let input_json = serde_json::to_string(items).unwrap_or_default();

        let req_body = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: input_json,
                },
            ],
            temperature: 0.1,
            response_format: Some(ResponseFormat {
                format_type: "json_object".to_string(),
            }),
        };

        let resp = self
            .client
            .post(&endpoint)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req_body)
            .send()
            .await
            .map_err(|e| super::super::EngineError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(super::super::EngineError::Api(format!(
                "OpenAI returned HTTP {}",
                resp.status()
            )));
        }

        let chat_res: ChatResponse = resp
            .json()
            .await
            .map_err(|e| super::super::EngineError::Json(e.to_string()))?;

        let first = chat_res
            .choices
            .first()
            .ok_or_else(|| super::super::EngineError::Api("Empty response from model".to_string()))?;

        let parsed: JsonTranslations = serde_json::from_str(&first.message.content)
            .map_err(|e| super::super::EngineError::Json(format!("Failed to parse JSON translations: {e}")))?;

        Ok(parsed.translations)
    }

    fn provider_name(&self) -> &'static str {
        "openai"
    }
}
