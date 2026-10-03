//! Local Ollama Translation Provider.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub struct OllamaProvider {
    pub base_url: String,
    pub model: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(base_url: Option<String>, model: Option<String>) -> Self {
        let url = base_url
            .or_else(|| std::env::var("OLLAMA_BASE_URL").ok())
            .unwrap_or_else(|| "http://localhost:11434".to_string());
        let mdl = model
            .or_else(|| std::env::var("OLLAMA_MODEL").ok())
            .unwrap_or_else(|| "qwen2.5:7b".to_string());

        Self {
            base_url: url,
            model: mdl,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    format: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

#[derive(Deserialize)]
struct JsonOutput {
    translations: Vec<String>,
}

#[async_trait]
impl super::super::TranslationProvider for OllamaProvider {
    async fn translate_batch(
        &self,
        items: &[String],
        target_locale: &str,
    ) -> Result<Vec<String>, super::super::EngineError> {
        let endpoint = format!("{}/api/generate", self.base_url.trim_end_matches('/'));

        let prompt = format!(
            "Translate this JSON array of strings into '{target_locale}'.\n\
            Preserve all placeholders like {{var}} and markup tags like <0>...</0> exactly.\n\
            Input: {}\n\
            Output format: {{ \"translations\": [\"...\"] }}",
            serde_json::to_string(items).unwrap_or_default()
        );

        let req_body = OllamaGenerateRequest {
            model: self.model.clone(),
            prompt,
            format: "json".to_string(),
            stream: false,
        };

        let resp = self
            .client
            .post(&endpoint)
            .json(&req_body)
            .send()
            .await
            .map_err(|e| super::super::EngineError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(super::super::EngineError::Api(format!(
                "Ollama returned HTTP {}",
                resp.status()
            )));
        }

        let body: OllamaGenerateResponse = resp
            .json()
            .await
            .map_err(|e| super::super::EngineError::Json(e.to_string()))?;

        let parsed: JsonOutput = serde_json::from_str(&body.response).map_err(|e| {
            super::super::EngineError::Json(format!("Failed to parse Ollama JSON: {e}"))
        })?;

        Ok(parsed.translations)
    }

    fn provider_name(&self) -> &'static str {
        "ollama"
    }
}
