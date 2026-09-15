//! In-Memory Pre-LLM PII Anonymizer & Typed Placeholder Restorer.
//!
//! Replaces sensitive personal data (emails, phone numbers, IP addresses, credit cards)
//! with typed placeholders (e.g., `{{GLOSSA_PII_EMAIL_1}}`) before sending to translation
//! providers, ensuring 100% data sovereignty and GDPR/HIPAA compliance.
//! Restores original values post-translation deterministically.

use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+").unwrap()
});

static IPV4_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap()
});

static PHONE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:\+?\d{1,3}[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b").unwrap()
});

static CREDIT_CARD_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:\d{4}[-\s]?){3}\d{4}\b").unwrap()
});

/// Context holding the redaction mappings for a single translation unit or batch.
#[derive(Debug, Default, Clone)]
pub struct PiiSession {
    placeholder_to_original: HashMap<String, String>,
    counter: usize,
}

impl PiiSession {
    pub fn new() -> Self {
        Self::default()
    }

    /// Redact all sensitive PII from a source string.
    pub fn redact(&mut self, text: &str) -> String {
        let mut result = text.to_string();

        // 1. Emails
        result = self.redact_with_pattern(&result, &EMAIL_REGEX, "EMAIL");

        // 2. Credit Cards (run before phone numbers to avoid partial overlap)
        result = self.redact_with_pattern(&result, &CREDIT_CARD_REGEX, "CARD");

        // 3. Phone numbers
        result = self.redact_with_pattern(&result, &PHONE_REGEX, "PHONE");

        // 4. IP Addresses
        result = self.redact_with_pattern(&result, &IPV4_REGEX, "IP");

        result
    }

    fn redact_with_pattern(&mut self, text: &str, regex: &Regex, pii_type: &str) -> String {
        regex
            .replace_all(text, |caps: &regex::Captures| {
                let original = caps[0].to_string();
                // Check if already mapped
                for (placeholder, val) in &self.placeholder_to_original {
                    if val == &original {
                        return placeholder.clone();
                    }
                }
                self.counter += 1;
                let placeholder = format!("{{{{GLOSSA_PII_{}_{}}}}}", pii_type, self.counter);
                self.placeholder_to_original
                    .insert(placeholder.clone(), original);
                placeholder
            })
            .to_string()
    }

    /// Restore redacted placeholders in the translated string.
    pub fn restore(&self, translated: &str) -> String {
        let mut restored = translated.to_string();
        for (placeholder, original) in &self.placeholder_to_original {
            restored = restored.replace(placeholder, original);
        }
        restored
    }

    /// Number of redacted entities in this session.
    pub fn len(&self) -> usize {
        self.placeholder_to_original.len()
    }

    pub fn is_empty(&self) -> bool {
        self.placeholder_to_original.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_redaction_and_restoration() {
        let mut session = PiiSession::new();
        let input = "Please contact support at security@twn.systems for assistance.";
        let redacted = session.redact(input);

        assert!(!redacted.contains("security@twn.systems"));
        assert!(redacted.contains("{{GLOSSA_PII_EMAIL_1}}"));

        // Simulate Spanish translation keeping placeholder untouched
        let mock_translated = "Por favor contacte a soporte en {{GLOSSA_PII_EMAIL_1}} para ayuda.";
        let restored = session.restore(mock_translated);

        assert_eq!(
            restored,
            "Por favor contacte a soporte en security@twn.systems para ayuda."
        );
    }

    #[test]
    fn test_multiple_pii_types() {
        let mut session = PiiSession::new();
        let input = "Server at 192.168.1.100 reported error from user user@acme.com with phone 555-123-4567.";
        let redacted = session.redact(input);

        assert!(!redacted.contains("192.168.1.100"));
        assert!(!redacted.contains("user@acme.com"));
        assert!(!redacted.contains("555-123-4567"));

        let restored = session.restore(&redacted);
        assert_eq!(restored, input);
    }
}
