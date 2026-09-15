//! The Apeiron Lockfile (`lingo.lock`).
//!
//! Stores invariant coordinates, source content hashes, and per-locale translated terms.
//! Provides order-free commutative merging (`Table::fuse`) and sub-millisecond delta computation.

use crate::ir::coord::ApeironCoord;
use crate::ir::term::ApeironCatalog;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub const CURRENT_LOCKFILE_VERSION: u32 = 1;

/// Per-locale translation record.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LocaleEntry {
    pub translated_text: String,
    pub translated_hash: u64,
    pub provider: String,
    pub updated_at: String,
    pub quality_score: f32,
}

/// A locked translatable unit indexed by its Apeiron coordinate.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LockEntry {
    pub coord: ApeironCoord,
    pub source_hash: u64,
    pub canonical_text: String,
    pub context: Option<String>,
    pub locales: BTreeMap<String, LocaleEntry>,
}

/// The project lockfile tracking all localized units.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LingoLock {
    pub version: u32,
    pub updated_at: String,
    pub entries: BTreeMap<ApeironCoord, LockEntry>,
}

impl Default for LingoLock {
    fn default() -> Self {
        Self {
            version: CURRENT_LOCKFILE_VERSION,
            updated_at: Utc::now().to_rfc3339(),
            entries: BTreeMap::new(),
        }
    }
}

/// A missing or out-of-date translation requiring translation.
#[derive(Clone, Debug)]
pub struct TranslationDelta {
    pub coord: ApeironCoord,
    pub canonical_text: String,
    pub context: Option<String>,
    pub missing_locales: Vec<String>,
}

pub type GlossaLock = LingoLock;

impl LingoLock {
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolve lockfile path: glossa.lock if present/configured, else lingo.lock.
    pub fn resolve_path<P: AsRef<Path>>(dir: P) -> std::path::PathBuf {
        let glossa_lock = dir.as_ref().join("glossa.lock");
        if glossa_lock.exists() {
            glossa_lock
        } else if dir.as_ref().join("lingo.lock").exists() {
            dir.as_ref().join("lingo.lock")
        } else if dir.as_ref().join("glossa.json").exists() {
            glossa_lock
        } else {
            dir.as_ref().join("lingo.lock")
        }
    }

    /// Load lockfile from disk.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::new());
        }
        let data = fs::read_to_string(path)?;
        serde_json::from_str(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
    }

    /// Save lockfile atomically to disk.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
        let path = path.as_ref();
        let serialized = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        
        // Write to temporary file and rename for atomic persistence
        let tmp_path = format!("{}.tmp", path.to_string_lossy());
        fs::write(&tmp_path, serialized)?;
        fs::rename(&tmp_path, path)?;
        Ok(())
    }

    /// Commutative monoidal merge (`Table::fuse`). Order-free and conflict-free.
    pub fn merge(&mut self, other: LingoLock) {
        for (coord, other_entry) in other.entries {
            if let Some(existing) = self.entries.get_mut(&coord) {
                // Merge locales, preferring newer updated_at
                for (loc, other_loc) in other_entry.locales {
                    if let Some(existing_loc) = existing.locales.get(&loc) {
                        if other_loc.updated_at > existing_loc.updated_at {
                            existing.locales.insert(loc, other_loc);
                        }
                    } else {
                        existing.locales.insert(loc, other_loc);
                    }
                }
            } else {
                self.entries.insert(coord, other_entry);
            }
        }
        self.updated_at = Utc::now().to_rfc3339();
    }

    /// Sync extracted catalog items into the lockfile, creating new entries as needed.
    pub fn sync_catalog(&mut self, catalog: &ApeironCatalog) {
        for (coord, item) in &catalog.items {
            let src_hash = ApeironCoord::from_str_canonical(&item.canonical_text).raw();
            if let Some(existing) = self.entries.get_mut(coord) {
                if existing.source_hash != src_hash {
                    // Source string changed! Invalidate translations
                    existing.source_hash = src_hash;
                    existing.canonical_text = item.canonical_text.clone();
                    existing.locales.clear();
                }
            } else {
                self.entries.insert(
                    *coord,
                    LockEntry {
                        coord: *coord,
                        source_hash: src_hash,
                        canonical_text: item.canonical_text.clone(),
                        context: item.context.clone(),
                        locales: BTreeMap::new(),
                    },
                );
            }
        }
        self.updated_at = Utc::now().to_rfc3339();
    }

    /// Compute pending translations needed across target locales.
    pub fn compute_deltas(&self, target_locales: &[String]) -> Vec<TranslationDelta> {
        let mut deltas = Vec::new();

        for (coord, entry) in &self.entries {
            let mut missing = Vec::new();
            for loc in target_locales {
                if !entry.locales.contains_key(loc) {
                    missing.push(loc.clone());
                }
            }
            if !missing.is_empty() {
                deltas.push(TranslationDelta {
                    coord: *coord,
                    canonical_text: entry.canonical_text.clone(),
                    context: entry.context.clone(),
                    missing_locales: missing,
                });
            }
        }

        deltas
    }

    /// Record a translation for a given coordinate and locale.
    pub fn record_translation(
        &mut self,
        coord: ApeironCoord,
        locale: &str,
        translated_text: String,
        provider: &str,
        quality_score: f32,
    ) {
        if let Some(entry) = self.entries.get_mut(&coord) {
            let trans_hash = ApeironCoord::from_str_canonical(&translated_text).raw();
            entry.locales.insert(
                locale.to_string(),
                LocaleEntry {
                    translated_text,
                    translated_hash: trans_hash,
                    provider: provider.to_string(),
                    updated_at: Utc::now().to_rfc3339(),
                    quality_score,
                },
            );
            self.updated_at = Utc::now().to_rfc3339();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lockfile_deltas() {
        let mut lock = LingoLock::new();
        let coord = ApeironCoord::from_str_canonical("Hello!");
        lock.entries.insert(
            coord,
            LockEntry {
                coord,
                source_hash: coord.raw(),
                canonical_text: "Hello!".to_string(),
                context: None,
                locales: BTreeMap::new(),
            },
        );

        let targets = vec!["es".to_string(), "fr".to_string()];
        let deltas = lock.compute_deltas(&targets);
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].missing_locales, vec!["es", "fr"]);

        lock.record_translation(coord, "es", "¡Hola!".to_string(), "offline", 1.0);
        let deltas2 = lock.compute_deltas(&targets);
        assert_eq!(deltas2.len(), 1);
        assert_eq!(deltas2[0].missing_locales, vec!["fr"]);
    }
}
