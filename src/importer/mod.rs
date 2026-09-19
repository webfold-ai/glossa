//! Universal i18n Importer & Drop-in Migration Engine.
//!
//! Automatically imports existing configurations and catalogs from:
//! - Lingo.dev (`i18n.json`/`i18n.lock` — current naming; `lingo.json`/`lingo.lock` — legacy)
//! - Inlang / Paraglide (`project.inlang/settings.json`)
//! - Tolgee (`tolgee.json`, `.tolgee`)
//! - Crowdin (`crowdin.yml`, `crowdin.yaml`)
//! - Lokalise (`lokalise.yml`, `lokalise.json`)
//! - i18next (`i18next-parser.config.js`, `next-i18next.config.js`)

use crate::config::LingoConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncumbentSystem {
    LingoDev,
    Inlang,
    Tolgee,
    Crowdin,
    Lokalise,
    I18next,
}

impl IncumbentSystem {
    pub fn name(&self) -> &'static str {
        match self {
            Self::LingoDev => "Lingo.dev",
            Self::Inlang => "Inlang / Paraglide",
            Self::Tolgee => "Tolgee",
            Self::Crowdin => "Crowdin",
            Self::Lokalise => "Lokalise",
            Self::I18next => "i18next",
        }
    }
}

pub struct DetectedImport {
    pub system: IncumbentSystem,
    pub config_file: PathBuf,
    pub source_locale: Option<String>,
    pub target_locales: Vec<String>,
    pub format: Option<String>,
}

/// Scans a directory to detect incumbent i18n configurations for drop-in migration.
pub fn detect_incumbent_systems(root: &Path) -> Vec<DetectedImport> {
    let mut detected = Vec::new();

    // 1. Check Lingo.dev — current real config is `i18n.json` (nested
    // `locale: {source, targets}` + `buckets: {<fmt>: {include}}` schema);
    // `lingo.json` is the legacy flat-field name kept for older projects.
    // Check `i18n.json` first so it isn't swallowed by the generic i18next
    // fallback further down, which used to mislabel it and discard the
    // real locale/target data sitting in the file.
    let i18n_json = root.join("i18n.json");
    let lingo_json = root.join("lingo.json");
    if i18n_json.exists() {
        let (src, targets, fmt) = parse_lingo_dev_i18n_json(&i18n_json);
        detected.push(DetectedImport {
            system: IncumbentSystem::LingoDev,
            config_file: i18n_json,
            source_locale: src,
            target_locales: targets,
            format: fmt,
        });
    } else if lingo_json.exists() {
        let (src, targets, fmt) = parse_lingo_json(&lingo_json);
        detected.push(DetectedImport {
            system: IncumbentSystem::LingoDev,
            config_file: lingo_json,
            source_locale: src,
            target_locales: targets,
            format: fmt,
        });
    }

    // 2. Check Inlang / Paraglide
    let inlang_dir = root.join("project.inlang");
    let inlang_settings = inlang_dir.join("settings.json");
    if inlang_settings.exists() {
        let (src, targets) = parse_inlang_settings(&inlang_settings);
        detected.push(DetectedImport {
            system: IncumbentSystem::Inlang,
            config_file: inlang_settings,
            source_locale: src,
            target_locales: targets,
            format: Some("json".to_string()),
        });
    }

    // 3. Check Tolgee
    let tolgee_json = root.join("tolgee.json");
    let tolgee_dot = root.join(".tolgee");
    if tolgee_json.exists() || tolgee_dot.exists() {
        let cfg_path = if tolgee_json.exists() { tolgee_json } else { tolgee_dot };
        detected.push(DetectedImport {
            system: IncumbentSystem::Tolgee,
            config_file: cfg_path,
            source_locale: Some("en".to_string()),
            target_locales: Vec::new(),
            format: Some("json".to_string()),
        });
    }

    // 4. Check Crowdin
    for name in &["crowdin.yml", "crowdin.yaml"] {
        let p = root.join(name);
        if p.exists() {
            let (src, targets) = parse_crowdin_yaml(&p);
            detected.push(DetectedImport {
                system: IncumbentSystem::Crowdin,
                config_file: p,
                source_locale: src,
                target_locales: targets,
                format: Some("yaml".to_string()),
            });
            break;
        }
    }

    // 5. Check Lokalise
    for name in &["lokalise.yml", "lokalise.yaml", "lokalise.json"] {
        let p = root.join(name);
        if p.exists() {
            detected.push(DetectedImport {
                system: IncumbentSystem::Lokalise,
                config_file: p,
                source_locale: Some("en".to_string()),
                target_locales: Vec::new(),
                format: Some("json".to_string()),
            });
            break;
        }
    }

    // 6. Check i18next. `i18n.json` is deliberately excluded here — it's
    // Lingo.dev's real config filename, handled in branch 1 above, and
    // must not fall through to this generic bucket a second time.
    for name in &["i18next-parser.config.js", "next-i18next.config.js"] {
        let p = root.join(name);
        if p.exists() {
            detected.push(DetectedImport {
                system: IncumbentSystem::I18next,
                config_file: p,
                source_locale: Some("en".to_string()),
                target_locales: Vec::new(),
                format: Some("json".to_string()),
            });
            break;
        }
    }

    detected
}

/// Performs drop-in migration: imports settings into GlossaConfig.
pub fn import_to_glossa_config(import: &DetectedImport, base_config: &mut LingoConfig) {
    if let Some(src) = &import.source_locale {
        base_config.source_locale = src.clone();
    }
    if !import.target_locales.is_empty() {
        for t in &import.target_locales {
            if !base_config.target_locales.contains(t) {
                base_config.target_locales.push(t.clone());
            }
        }
    }
    if let Some(fmt) = &import.format {
        base_config.output_format = fmt.clone();
    }
}

/// Parses Lingo.dev's real, current `i18n.json` schema:
/// `{ "locale": { "source": "en", "targets": [...] }, "buckets": { "<fmt>": { "include": [...] } } }`.
/// This is NOT the same shape as the legacy flat `sourceLocale`/`targetLocales`
/// fields `parse_lingo_json` below reads — Lingo.dev nests locale info under
/// `locale`, and the format is inferred from whichever key `buckets` uses
/// (real-world example, from `gitroomhq/postiz-app`'s `i18n.json`: `buckets.json`).
fn parse_lingo_dev_i18n_json(path: &Path) -> (Option<String>, Vec<String>, Option<String>) {
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
            let locale = v.get("locale");
            let src = locale
                .and_then(|l| l.get("source"))
                .and_then(|s| s.as_str())
                .map(|s| s.to_string());
            let targets = locale
                .and_then(|l| l.get("targets"))
                .and_then(|a| a.as_array())
                .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            let fmt = v
                .get("buckets")
                .and_then(|b| b.as_object())
                .and_then(|m| m.keys().next())
                .map(|k| k.to_string());
            return (src, targets, fmt);
        }
    }
    (None, Vec::new(), None)
}

fn parse_lingo_json(path: &Path) -> (Option<String>, Vec<String>, Option<String>) {
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
            let src = v.get("sourceLocale").and_then(|s| s.as_str()).map(|s| s.to_string());
            let targets = v.get("targetLocales")
                .and_then(|a| a.as_array())
                .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            let fmt = v.get("format").and_then(|s| s.as_str()).map(|s| s.to_string());
            return (src, targets, fmt);
        }
    }
    (None, Vec::new(), None)
}

fn parse_inlang_settings(path: &Path) -> (Option<String>, Vec<String>) {
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
            let src = v.get("sourceLanguageTag").and_then(|s| s.as_str()).map(|s| s.to_string());
            let targets = v.get("languageTags")
                .and_then(|a| a.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .filter(|t| Some(t.as_str()) != src.as_deref())
                        .collect()
                })
                .unwrap_or_default();
            return (src, targets);
        }
    }
    (None, Vec::new())
}

fn parse_crowdin_yaml(path: &Path) -> (Option<String>, Vec<String>) {
    let mut src = None;
    let mut targets = Vec::new();
    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("source_language:") {
                src = trimmed.split(':').nth(1).map(|s| s.trim().trim_matches('"').to_string());
            } else if trimmed.starts_with("languages:") {
                // Parse simple array format
            } else if trimmed.starts_with("- ") && targets.len() < 20 {
                let loc = trimmed.trim_start_matches("- ").trim().trim_matches('"');
                if loc.len() == 2 || loc.contains('-') || loc.contains('_') {
                    targets.push(loc.to_string());
                }
            }
        }
    }
    (src, targets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_detect_lingo_import() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        let lingo_content = r#"
        {
            "sourceLocale": "en",
            "targetLocales": ["es", "fr", "ja"],
            "format": "json"
        }
        "#;
        fs::write(root.join("lingo.json"), lingo_content).unwrap();

        let imports = detect_incumbent_systems(root);
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].system, IncumbentSystem::LingoDev);
        assert_eq!(imports[0].source_locale.as_deref(), Some("en"));
        assert_eq!(imports[0].target_locales, vec!["es", "fr", "ja"]);
    }

    /// Real fixture: verbatim `i18n.json` content pulled from
    /// `gitroomhq/postiz-app` (a live, 36k-star Lingo.dev user) on 2026-09-19.
    /// Regression guard for the bug where glossa only recognized the legacy
    /// flat `lingo.json` shape and silently mis-detected real Lingo.dev
    /// projects as generic "i18next" with a hardcoded empty target list.
    #[test]
    fn test_detect_lingo_dev_i18n_json_real_world_shape() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        let i18n_json = r#"
        {
          "version": 1.8,
          "provider": {
            "id": "openai",
            "model": "gpt-4.1",
            "prompt": "Translate accurately from {source} to {target}, maintaining the original meaning.",
            "baseUrl": "https://api.openai.com/v1"
          },
          "locale": {
            "source": "en",
            "targets": ["he", "ru", "zh", "fr", "bn", "es", "pt", "de", "it", "ja", "ko", "ar", "tr", "vi"]
          },
          "buckets": {
            "json": {
              "include": ["libraries/react-shared-libraries/src/translation/locales/[locale]/translation.json"]
            }
          },
          "$schema": "https://lingo.dev/schema/i18n.json"
        }
        "#;
        fs::write(root.join("i18n.json"), i18n_json).unwrap();

        let imports = detect_incumbent_systems(root);
        assert_eq!(imports.len(), 1, "must detect exactly one system, not double-count via the i18next fallback");
        assert_eq!(imports[0].system, IncumbentSystem::LingoDev, "real i18n.json must classify as LingoDev, not I18next");
        assert_eq!(imports[0].source_locale.as_deref(), Some("en"));
        assert_eq!(imports[0].target_locales.len(), 14, "must read the real 14-locale target list, not default to empty");
        assert!(imports[0].target_locales.contains(&"ja".to_string()));
        assert_eq!(imports[0].format.as_deref(), Some("json"));
    }

    /// `i18n.json` takes priority over legacy `lingo.json` when both exist,
    /// and must not be detected twice.
    #[test]
    fn test_i18n_json_takes_priority_over_legacy_lingo_json() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        fs::write(
            root.join("i18n.json"),
            r#"{"locale": {"source": "en", "targets": ["es"]}, "buckets": {"json": {"include": []}}}"#,
        )
        .unwrap();
        fs::write(
            root.join("lingo.json"),
            r#"{"sourceLocale": "en", "targetLocales": ["de", "fr"], "format": "json"}"#,
        )
        .unwrap();

        let imports = detect_incumbent_systems(root);
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].target_locales, vec!["es"]);
    }

    #[test]
    fn test_detect_inlang_import() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        let inlang_dir = root.join("project.inlang");
        fs::create_dir_all(&inlang_dir).unwrap();
        let inlang_content = r#"
        {
            "sourceLanguageTag": "en",
            "languageTags": ["en", "de", "zh-CN"]
        }
        "#;
        fs::write(inlang_dir.join("settings.json"), inlang_content).unwrap();

        let imports = detect_incumbent_systems(root);
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].system, IncumbentSystem::Inlang);
        assert_eq!(imports[0].target_locales, vec!["de", "zh-CN"]);
    }
}
