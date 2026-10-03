//! Translation Integrity Gate.
//!
//! Validates that translations preserve strict structural invariants.
//! If an LLM drops a variable, renames a slot, or breaks markup tags,
//! the translation hits the integrity gate and is rejected immediately.

use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum GateViolation {
    #[error("Missing slot placeholder: '{slot}' was present in source but missing in translation")]
    MissingSlot { slot: String },

    #[error("Hallucinated or extraneous slot placeholder: '{slot}' in translation")]
    ExtraneousSlot { slot: String },

    #[error("Mismatched or unclosed markup tag: '<{tag}>' in translation")]
    UnclosedTag { tag: String },

    #[error(
        "Security violation: potential script or dangerous HTML injection detected in translation"
    )]
    SecurityRisk { reason: String },

    #[error(
        "Glossary violation: term '{term}' was expected to be translated/preserved as '{expected}'"
    )]
    GlossaryViolation { term: String, expected: String },
}

/// Zero-fault translation integrity validation gate.
pub struct ApeironNullGate;

/// Alias for translation integrity gate.
pub type IntegrityGate = ApeironNullGate;

impl ApeironNullGate {
    /// Validates translation against source for slot conservation and markup safety.
    pub fn validate(source_canonical: &str, translated: &str) -> Result<(), GateViolation> {
        Self::validate_with_glossary(source_canonical, translated, None)
    }

    /// Validates translation against source for slot conservation, markup safety, and protected glossary keywords.
    pub fn validate_with_glossary(
        source_canonical: &str,
        translated: &str,
        glossary: Option<&std::collections::BTreeMap<String, String>>,
    ) -> Result<(), GateViolation> {
        // 1. Check for dangerous HTML injections
        let lower = translated.to_ascii_lowercase();
        if lower.contains("<script")
            || lower.contains("javascript:")
            || lower.contains("onerror=")
            || lower.contains("onload=")
        {
            return Err(GateViolation::SecurityRisk {
                reason: "Dangerous script or attribute found".to_string(),
            });
        }

        // 2. Extract slots from source and translation
        let src_slots = extract_slots(source_canonical);
        let trans_slots = extract_slots(translated);

        for slot in &src_slots {
            if !trans_slots.contains(slot) {
                return Err(GateViolation::MissingSlot { slot: slot.clone() });
            }
        }

        for slot in &trans_slots {
            if !src_slots.contains(slot) {
                return Err(GateViolation::ExtraneousSlot { slot: slot.clone() });
            }
        }

        // 3. Check markup tag balancing (<0>...</0>)
        let src_tags = extract_numbered_tags(source_canonical);
        let trans_tags = extract_numbered_tags(translated);

        if src_tags != trans_tags {
            return Err(GateViolation::UnclosedTag {
                tag: format!("Mismatched tag structure: expected {src_tags:?}, got {trans_tags:?}"),
            });
        }

        // 4. Check protected glossary brand terms if configured
        if let Some(gloss) = glossary {
            for (term, expected) in gloss {
                if source_canonical.contains(term) && !translated.contains(expected) {
                    return Err(GateViolation::GlossaryViolation {
                        term: term.clone(),
                        expected: expected.clone(),
                    });
                }
            }
        }

        Ok(())
    }
}

/// Extracts all `{var}` slot names.
fn extract_slots(s: &str) -> Vec<String> {
    let mut slots = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '{' {
            i += 1;
            let mut name = String::new();
            while i < chars.len() && chars[i] != '}' && chars[i] != ',' {
                name.push(chars[i]);
                i += 1;
            }
            while i < chars.len() && chars[i] != '}' {
                i += 1;
            }
            if i < chars.len() {
                i += 1;
            }
            let trimmed = name.trim().to_string();
            if !trimmed.is_empty() {
                slots.push(trimmed);
            }
        } else {
            i += 1;
        }
    }
    slots.sort();
    slots
}

/// Extracts list of `<N>` and `</N>` tags to verify parity.
fn extract_numbered_tags(s: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '<' {
            i += 1;
            let mut tag = String::new();
            while i < chars.len() && chars[i] != '>' {
                tag.push(chars[i]);
                i += 1;
            }
            if i < chars.len() {
                i += 1;
            }
            let trimmed = tag.trim().to_string();
            if trimmed.chars().all(|c| c.is_ascii_digit() || c == '/') {
                tags.push(trimmed);
            }
        } else {
            i += 1;
        }
    }
    tags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_valid() {
        let src = "Hello {name}, click <0>here</0> to proceed.";
        let trans = "¡Hola {name}, haz clic <0>aquí</0> para continuar.";
        assert!(ApeironNullGate::validate(src, trans).is_ok());
    }

    #[test]
    fn test_gate_missing_slot() {
        let src = "Hello {name}, you have {count} items.";
        let trans = "Hola {name}, tienes artículos.";
        let err = ApeironNullGate::validate(src, trans).unwrap_err();
        assert_eq!(
            err,
            GateViolation::MissingSlot {
                slot: "count".to_string()
            }
        );
    }

    #[test]
    fn test_gate_mismatched_tags() {
        let src = "Click <0>here</0> to subscribe.";
        let trans = "Haga clic <0>aquí para suscribirse.";
        let err = ApeironNullGate::validate(src, trans).unwrap_err();
        assert!(matches!(err, GateViolation::UnclosedTag { .. }));
    }

    #[test]
    fn test_gate_security_risk() {
        let src = "Welcome!";
        let trans = "Welcome!<script>alert(1)</script>";
        let err = ApeironNullGate::validate(src, trans).unwrap_err();
        assert!(matches!(err, GateViolation::SecurityRisk { .. }));
    }
}
