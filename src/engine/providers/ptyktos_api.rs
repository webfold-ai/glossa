//! Ptyktos API Translation Provider.
//!
//! Connects to Ptyktos / Tasmanian Cloud / Bedrock headless research and reasoning backend.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub struct PtyktosApiProvider {
    pub base_url: String,
    pub api_key: Option<String>,
    client: reqwest::Client,
}

impl PtyktosApiProvider {
    pub fn new(base_url: Option<String>, api_key: Option<String>) -> Self {
        let url = base_url
            .or_else(|| std::env::var("PTYKTOS_API_URL").ok())
            .unwrap_or_else(|| "http://127.0.0.1:8080".to_string());
        let key = api_key.or_else(|| std::env::var("PTYKTOS_API_KEY").ok());

        Self {
            base_url: url,
            api_key: key,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct TranslateRequest<'a> {
    texts: &'a [String],
    target_locale: &'a str,
}

#[derive(Deserialize)]
struct TranslateResponse {
    translations: Vec<String>,
}

#[async_trait]
impl super::super::TranslationProvider for PtyktosApiProvider {
    async fn translate_batch(
        &self,
        items: &[String],
        target_locale: &str,
    ) -> Result<Vec<String>, super::super::EngineError> {
        let endpoint = format!("{}/v1/lingo/translate", self.base_url);
        let mut req = self.client.post(&endpoint).json(&TranslateRequest {
            texts: items,
            target_locale,
        });

        if let Some(key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {key}"));
        }

        let resp = req
            .send()
            .await
            .map_err(|e| super::super::EngineError::Http(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(super::super::EngineError::Api(format!(
                "Ptyktos API returned HTTP {}",
                resp.status()
            )));
        }

        let body: TranslateResponse = resp
            .json()
            .await
            .map_err(|e| super::super::EngineError::Json(e.to_string()))?;

        Ok(body.translations)
    }

    fn provider_name(&self) -> &'static str {
        "ptyktos-api"
    }
}
