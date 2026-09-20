//! The Translation Engine & Provider Management.

pub mod gate;
pub mod locale_map;
pub mod lorelake;
pub mod pii;
pub mod providers;
pub mod pseudo;
pub mod tm;

use async_trait::async_trait;
use gate::{ApeironNullGate, GateViolation};
pub use pii::PiiSession;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("HTTP transport error: {0}")]
    Http(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("JSON error: {0}")]
    Json(String),

    #[error("Gate violation: {0}")]
    Gate(#[from] GateViolation),
}

#[async_trait]
pub trait TranslationProvider: Send + Sync {
    /// Translate a batch of canonical text items into the target locale.
    async fn translate_batch(
        &self,
        items: &[String],
        target_locale: &str,
    ) -> Result<Vec<String>, EngineError>;

    /// Human-readable provider identifier.
    fn provider_name(&self) -> &'static str;
}

/// Orchestrates batching, provider calls, and Apeiron null-gate verification.
pub struct TranslationEngine {
    provider: Arc<dyn TranslationProvider>,
}

impl TranslationEngine {
    pub fn new(provider: Arc<dyn TranslationProvider>) -> Self {
        Self { provider }
    }

    /// Translate a slice of source canonical strings, enforcing in-memory PII redaction
    /// and the Apeiron Null-Gate.
    pub async fn translate_and_verify(
        &self,
        sources: &[String],
        target_locale: &str,
    ) -> Result<Vec<String>, EngineError> {
        // 1. Redact PII in memory before dispatching to external LLMs
        let mut pii_sessions: Vec<PiiSession> = Vec::with_capacity(sources.len());
        let mut redacted_sources: Vec<String> = Vec::with_capacity(sources.len());
        for src in sources {
            let mut session = PiiSession::new();
            let redacted = session.redact(src);
            pii_sessions.push(session);
            redacted_sources.push(redacted);
        }

        let raw_translations = self.provider.translate_batch(&redacted_sources, target_locale).await?;

        if raw_translations.len() != sources.len() {
            return Err(EngineError::Api(format!(
                "Provider returned {} items for {} inputs",
                raw_translations.len(),
                sources.len()
            )));
        }

        let mut verified = Vec::with_capacity(raw_translations.len());
        for ((src, trans), pii) in sources.iter().zip(raw_translations).zip(pii_sessions) {
            // 2. Restore exact PII values post-translation
            let restored = pii.restore(&trans);
            // 3. Validate through the Apeiron Null-Gate
            ApeironNullGate::validate(src, &restored)?;
            verified.push(restored);
        }

        Ok(verified)
    }

    pub fn provider_name(&self) -> &'static str {
        self.provider.provider_name()
    }
}
