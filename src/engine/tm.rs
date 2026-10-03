//! Translation Memory (TM) & Fuzzy Matching Engine.
//!
//! Grounded in Apeiron's invariant metric space:
//! Calculates similarity scores (0.0 to 1.0) between source strings using Levenshtein distance
//! and n-gram Jaccard coefficients.
//! Delivers high-speed reuse across projects, saving up to 70% of LLM translation cost and latency.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A translation candidate retrieved from the Translation Memory.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TmCandidate {
    pub source_text: String,
    pub translated_text: String,
    pub locale: String,
    pub similarity_score: f64,
    pub match_category: TmMatchCategory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TmMatchCategory {
    ExactMatch,       // 100%
    HighFuzzyMatch,   // 85% - 99%
    MediumFuzzyMatch, // 70% - 84%
    LowFuzzyMatch,    // 50% - 69%
    NoMatch,          // < 50%
}

impl TmMatchCategory {
    pub fn from_score(score: f64) -> Self {
        if score >= 0.999 {
            Self::ExactMatch
        } else if score >= 0.85 {
            Self::HighFuzzyMatch
        } else if score >= 0.70 {
            Self::MediumFuzzyMatch
        } else if score >= 0.50 {
            Self::LowFuzzyMatch
        } else {
            Self::NoMatch
        }
    }
}

/// In-memory high-speed Translation Memory store.
#[derive(Clone, Debug, Default)]
pub struct TranslationMemory {
    /// Maps locale -> (source_text -> translated_text)
    entries: BTreeMap<String, BTreeMap<String, String>>,
}

impl TranslationMemory {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Add a translation unit into the memory.
    pub fn insert(&mut self, locale: &str, source: &str, translation: &str) {
        let loc_map = self.entries.entry(locale.to_string()).or_default();
        loc_map.insert(source.to_string(), translation.to_string());
    }

    /// Find the best matching candidate for a given source string in a target locale.
    pub fn find_best_match(
        &self,
        locale: &str,
        query: &str,
        min_similarity: f64,
    ) -> Option<TmCandidate> {
        let loc_map = self.entries.get(locale)?;
        let mut best_score = 0.0;
        let mut best_entry = None;

        for (src, trans) in loc_map {
            let score = compute_similarity(query, src);
            if score > best_score && score >= min_similarity {
                best_score = score;
                best_entry = Some((src, trans));
            }
        }

        best_entry.map(|(src, trans)| TmCandidate {
            source_text: src.clone(),
            translated_text: trans.clone(),
            locale: locale.to_string(),
            similarity_score: best_score,
            match_category: TmMatchCategory::from_score(best_score),
        })
    }

    /// Retrieve top N fuzzy matching candidates.
    pub fn find_top_matches(
        &self,
        locale: &str,
        query: &str,
        top_k: usize,
        min_similarity: f64,
    ) -> Vec<TmCandidate> {
        let mut candidates = Vec::new();
        if let Some(loc_map) = self.entries.get(locale) {
            for (src, trans) in loc_map {
                let score = compute_similarity(query, src);
                if score >= min_similarity {
                    candidates.push(TmCandidate {
                        source_text: src.clone(),
                        translated_text: trans.clone(),
                        locale: locale.to_string(),
                        similarity_score: score,
                        match_category: TmMatchCategory::from_score(score),
                    });
                }
            }
        }

        candidates.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.truncate(top_k);
        candidates
    }

    pub fn len(&self, locale: &str) -> usize {
        self.entries.get(locale).map(|m| m.len()).unwrap_or(0)
    }
}

/// Computes normalized similarity between two strings (0.0 to 1.0)
/// combining Levenshtein distance and character trigram overlap.
pub fn compute_similarity(s1: &str, s2: &str) -> f64 {
    if s1 == s2 {
        return 1.0;
    }
    if s1.is_empty() || s2.is_empty() {
        return 0.0;
    }

    let lev_dist = levenshtein_distance(s1, s2);
    let max_len = s1.chars().count().max(s2.chars().count());
    let lev_sim = 1.0 - (lev_dist as f64 / max_len as f64);

    let jaccard_sim = trigram_jaccard(s1, s2);

    // Weighted blend: 60% Levenshtein, 40% Trigram
    0.6 * lev_sim + 0.4 * jaccard_sim
}

/// Standard Levenshtein edit distance algorithm.
pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let chars1: Vec<char> = s1.chars().collect();
    let chars2: Vec<char> = s2.chars().collect();
    let m = chars1.len();
    let n = chars2.len();

    let mut dp = vec![vec![0; n + 1]; m + 1];

    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }

    dp[m][n]
}

/// Character 3-gram Jaccard coefficient.
fn trigram_jaccard(s1: &str, s2: &str) -> f64 {
    let t1 = get_trigrams(s1);
    let t2 = get_trigrams(s2);

    if t1.is_empty() && t2.is_empty() {
        return 1.0;
    }
    if t1.is_empty() || t2.is_empty() {
        return 0.0;
    }

    let mut intersection = 0;
    for g in &t1 {
        if t2.contains(g) {
            intersection += 1;
        }
    }

    let union = t1.len() + t2.len() - intersection;
    if union == 0 {
        1.0
    } else {
        intersection as f64 / union as f64
    }
}

fn get_trigrams(s: &str) -> Vec<String> {
    let chars: Vec<char> = s.to_lowercase().chars().collect();
    if chars.len() < 3 {
        return vec![s.to_lowercase()];
    }
    let mut out = Vec::with_capacity(chars.len() - 2);
    for i in 0..=chars.len() - 3 {
        let tri: String = chars[i..i + 3].iter().collect();
        out.push(tri);
    }
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity_exact() {
        let s = "Hello, world!";
        assert_eq!(compute_similarity(s, s), 1.0);
    }

    #[test]
    fn test_similarity_fuzzy() {
        let s1 = "Please enter your email address to continue.";
        let s2 = "Please enter your billing address to continue.";
        let sim = compute_similarity(s1, s2);
        assert!(sim > 0.75, "Expected high fuzzy similarity, got {sim}");
    }

    #[test]
    fn test_tm_candidate_lookup() {
        let mut tm = TranslationMemory::new();
        tm.insert(
            "es",
            "Please confirm your password before submitting.",
            "Por favor, confirme su contraseña antes de enviar.",
        );

        let query = "Please confirm your email before submitting.";
        let candidate = tm
            .find_best_match("es", query, 0.7)
            .expect("Must find fuzzy match");
        assert!(candidate.similarity_score > 0.70);
        assert!(candidate.translated_text.contains("confirme su contraseña"));
    }
}
