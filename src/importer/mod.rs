//! Universal i18n Importer & Drop-in Migration Engine.
//!
//! Automatically imports existing configurations and catalogs from:
//! - Lingo.dev (`lingo.json`, `lingo.lock`)
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

    // 1. Check Lingo.dev
    let lingo_json = root.join("lingo.json");
    if lingo_json.exists() {
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

    // 6. Check i18next
    for name in &["i18next-parser.config.js", "next-i18next.config.js", "i18n.json"] {
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
