//! Universal i18n Importer & Drop-in Migration Engine.
//!
//! Automatically imports existing configurations and catalogs from:
//! - Lingo.dev (`i18n.json`/`i18n.lock` — current naming; `lingo.json`/`lingo.lock` — legacy)
//! - Inlang / Paraglide (`project.inlang/settings.json`)
//! - Tolgee (`.tolgeerc`/`.tolgeerc.json` and variants, `tolgee.config.{js,cjs}`)
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
    /// Source-file glob patterns pulled from the incumbent's own config
    /// (Tolgee's `patterns`, Crowdin's `files[].source`). Real-world Tolgee
    /// and Crowdin configs manage target locales on the platform side, not
    /// in this file, so these patterns are often the only concrete signal
    /// actually available locally.
    pub include_patterns: Vec<String>,
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
            include_patterns: Vec::new(),
        });
    } else if lingo_json.exists() {
        let (src, targets, fmt) = parse_lingo_json(&lingo_json);
        detected.push(DetectedImport {
            system: IncumbentSystem::LingoDev,
            config_file: lingo_json,
            source_locale: src,
            target_locales: targets,
            format: fmt,
            include_patterns: Vec::new(),
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
            include_patterns: Vec::new(),
        });
    }

    // 3. Check Tolgee. Real Tolgee CLI configs (verified against
    // satisfactorymodding/SatisfactoryModManager and Tolgee's own
    // webapp/.tolgeerc.json) are named `.tolgeerc`/`.tolgeerc.json` (or the
    // `.yaml`/`.yml`/`.js`/`.cjs` variants, or `tolgee.config.{js,cjs}`) —
    // never `tolgee.json` or bare `.tolgee`, which this used to check for
    // exclusively and so never matched a single real project. Real configs
    // also carry no source/target locale list at all (that's managed on the
    // Tolgee platform, keyed by `projectId`) — but they do carry `patterns`,
    // the actual source-file globs, which is real signal worth keeping.
    for name in &[
        ".tolgeerc",
        ".tolgeerc.json",
        ".tolgeerc.yaml",
        ".tolgeerc.yml",
        ".tolgeerc.js",
        ".tolgeerc.cjs",
        "tolgee.config.js",
        "tolgee.config.cjs",
    ] {
        let p = root.join(name);
        if p.exists() {
            let patterns = parse_tolgeerc_patterns(&p);
            detected.push(DetectedImport {
                system: IncumbentSystem::Tolgee,
                config_file: p,
                source_locale: Some("en".to_string()),
                target_locales: Vec::new(),
                format: Some("json".to_string()),
                include_patterns: patterns,
            });
            break;
        }
    }

    // 4. Check Crowdin. Real crowdin.yml files (verified against
    // DS-Homebrew/TWiLightMenu and mendhak/gpslogger) declare a `files:`
    // source/translation path mapping, not a `source_language:`/`languages:`
    // list — those keys are kept as a fallback for configs that do set them,
    // but `files[].source` is the concrete signal actually present in the
    // common case.
    for name in &["crowdin.yml", "crowdin.yaml"] {
        let p = root.join(name);
        if p.exists() {
            let (src, targets, patterns) = parse_crowdin_yaml(&p);
            detected.push(DetectedImport {
                system: IncumbentSystem::Crowdin,
                config_file: p,
                source_locale: src,
                target_locales: targets,
                format: Some("yaml".to_string()),
                include_patterns: patterns,
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
                include_patterns: Vec::new(),
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
                include_patterns: Vec::new(),
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
    for pattern in &import.include_patterns {
        if !base_config.include.contains(pattern) {
            base_config.include.push(pattern.clone());
        }
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

/// Inlang has two real, currently-active `project.inlang/settings.json`
/// schemas in the wild, not one:
/// - Legacy (`@inlang/plugin-i18next`): `sourceLanguageTag`/`languageTags`.
///   Verified against `AppFlowy-IO/AppFlowy`.
/// - Current (`@inlang/plugin-message-format`): `baseLocale`/`locales`.
///   Verified against two independent real projects, `VERT-sh/VERT` and
///   `Splode/pomotroid` — both live, active, and using the newer schema.
/// Before this fix, only the legacy field names were read, so both real
/// message-format projects silently returned no source/target locales at
/// all (the file was still detected, since detection only checks the path
/// exists, but the actual locale data glossa needs was dropped).
fn parse_inlang_settings(path: &Path) -> (Option<String>, Vec<String>) {
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
            let src = v
                .get("sourceLanguageTag")
                .or_else(|| v.get("baseLocale"))
                .and_then(|s| s.as_str())
                .map(|s| s.to_string());
            let targets = v
                .get("languageTags")
                .or_else(|| v.get("locales"))
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

/// Parses a real Crowdin `crowdin.yml`. The common real-world shape (verified
/// against `DS-Homebrew/TWiLightMenu` and `mendhak/gpslogger`) is a `files:`
/// list of `source:`/`translation:` path pairs with no `source_language:` or
/// `languages:` keys at all — those are kept below for configs that do set
/// them, but `files[].source` is the signal actually present in practice.
fn parse_crowdin_yaml(path: &Path) -> (Option<String>, Vec<String>, Vec<String>) {
    let mut src = None;
    let mut targets = Vec::new();
    let mut sources = Vec::new();
    let mut in_languages = false;
    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("source_language:") {
                src = Some(rest.trim().trim_matches('"').to_string());
                in_languages = false;
            } else if trimmed.starts_with("languages:") {
                in_languages = true;
            } else if let Some(rest) = trimmed.strip_prefix("- source:") {
                sources.push(rest.trim().trim_matches('"').to_string());
                in_languages = false;
            } else if trimmed.starts_with("- ") && in_languages && targets.len() < 20 {
                let loc = trimmed.trim_start_matches("- ").trim().trim_matches('"');
                if loc.len() == 2 || loc.contains('-') || loc.contains('_') {
                    targets.push(loc.to_string());
                }
            } else if !trimmed.starts_with('-') && !trimmed.is_empty() {
                in_languages = false;
            }
        }
    }
    (src, targets, sources)
}

/// Parses the `patterns` array (source-file globs) out of a real Tolgee
/// `.tolgeerc.json`. Only the JSON variants are parsed here — `.tolgeerc.js`/
/// `.tolgeerc.cjs`/`tolgee.config.{js,cjs}` are detected by filename only,
/// since they're executable JS, not data.
fn parse_tolgeerc_patterns(path: &Path) -> Vec<String> {
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
            return v
                .get("patterns")
                .and_then(|a| a.as_array())
                .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
        }
    }
    Vec::new()
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

    /// Third independent real fixture — `formbricks/formbricks` (12.9k
    /// stars), confirming the schema generalizes and exercising
    /// region-tagged locale codes (`zh-Hans-CN`, and two distinct Portuguese
    /// variants `pt-BR`/`pt-PT`) that the postiz-app fixture didn't cover.
    /// Real path is `apps/web/i18n.json` in formbricks' monorepo, not the
    /// repo root — this fixture validates as if glossa were pointed at that
    /// subpackage directory, which is the realistic invocation for a
    /// per-package scan; scanning an entire monorepo root for configs nested
    /// under `apps/*`/`packages/*` is a separate, larger question not
    /// covered by this fix.
    #[test]
    fn test_detect_lingo_dev_i18n_json_real_world_shape_formbricks() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        fs::write(
            root.join("i18n.json"),
            r#"{
              "$schema": "https://lingo.dev/schema/i18n.json",
              "buckets": { "json": { "include": ["locales/[locale].json"] } },
              "locale": {
                "source": "en-US",
                "targets": ["de-DE", "es-ES", "fr-FR", "hu-HU", "ja-JP", "nl-NL", "pt-BR", "pt-PT", "ro-RO", "ru-RU", "sv-SE", "tr-TR", "zh-Hans-CN", "zh-Hant-TW"]
              },
              "version": 1.8
            }"#,
        )
        .unwrap();

        let imports = detect_incumbent_systems(root);
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].system, IncumbentSystem::LingoDev);
        assert_eq!(imports[0].source_locale.as_deref(), Some("en-US"), "region-tagged source locale must survive intact");
        assert_eq!(imports[0].target_locales.len(), 14);
        assert!(imports[0].target_locales.contains(&"pt-BR".to_string()));
        assert!(imports[0].target_locales.contains(&"pt-PT".to_string()), "two distinct Portuguese variants must both be kept, not deduped");
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

    /// Real fixture: verbatim `.tolgeerc.json` from
    /// `satisfactorymodding/SatisfactoryModManager` (a real, live Tolgee
    /// user). Regression guard for the bug where glossa only checked for
    /// `tolgee.json`/`.tolgee` — filenames that don't exist in any real
    /// Tolgee project — so Tolgee was never detected at all.
    #[test]
    fn test_detect_tolgee_real_world_shape_satisfactory_mod_manager() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        let tolgeerc = r#"
        {
          "$schema": "https://tolgee.io/cli-schema.json",
          "projectId": 4,
          "apiUrl": "https://translate.ficsit.app",
          "patterns": ["./src/**/*.ts?(x)", "./src/**/*.svelte"],
          "pull": { "path": "./src/lib/generated/i18n" }
        }
        "#;
        fs::write(root.join(".tolgeerc.json"), tolgeerc).unwrap();

        let imports = detect_incumbent_systems(root);
        assert_eq!(imports.len(), 1, "must detect Tolgee from the real .tolgeerc.json filename");
        assert_eq!(imports[0].system, IncumbentSystem::Tolgee);
        assert_eq!(
            imports[0].include_patterns,
            vec!["./src/**/*.ts?(x)", "./src/**/*.svelte"],
            "must pull the real source-file patterns, the only concrete signal a real Tolgee config carries locally"
        );
    }

    /// Second independent real fixture — Tolgee's own dogfooding config at
    /// `tolgee/tolgee-platform`'s `webapp/.tolgeerc.json`. Same shape family,
    /// different project, confirming this isn't a one-off coincidence.
    #[test]
    fn test_detect_tolgee_real_world_shape_tolgee_platform_dogfood() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        fs::write(
            root.join(".tolgeerc.json"),
            r#"{"$schema": "https://docs.tolgee.io/cli-schema.json", "projectId": 1, "patterns": ["./src/**/*.ts?(x)"], "strictNamespace": false, "pull": {"path": "./src/i18n"}}"#,
        )
        .unwrap();

        let imports = detect_incumbent_systems(root);
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].system, IncumbentSystem::Tolgee);
        assert_eq!(imports[0].include_patterns, vec!["./src/**/*.ts?(x)"]);
    }

    /// Real fixture: `watt-mind/coach`'s `.tolgeerc` (no `.json` extension) —
    /// a third distinct real shape with no `patterns` field at all, only
    /// `push.files`/`pull`. Confirms detection and the absence of a crash
    /// when `patterns` is missing (include_patterns should just be empty,
    /// not an error) rather than asserting a bug fix.
    #[test]
    fn test_detect_tolgee_real_world_shape_watt_mind_coach_no_patterns() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        fs::write(
            root.join(".tolgeerc"),
            r#"{"projectId": 2, "push": {"files": [{"path": "app/i18n/en", "language": "en", "template": "{namespace}.json"}]}, "pull": {"path": "app/i18n", "fileStructureTemplate": "{languageTag}/{namespace}.json"}}"#,
        )
        .unwrap();

        let imports = detect_incumbent_systems(root);
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].system, IncumbentSystem::Tolgee);
        assert!(imports[0].include_patterns.is_empty(), "no `patterns` field in this real shape — must be empty, not a parse error");
    }

    /// Real fixture: `liquidinvestigations/hoover-ui`'s `.tolgeerc` — the
    /// minimal real-world extreme, just `{"projectId": 3827}`.
    #[test]
    fn test_detect_tolgee_real_world_shape_minimal_hoover_ui() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        fs::write(root.join(".tolgeerc"), r#"{"projectId": 3827}"#).unwrap();

        let imports = detect_incumbent_systems(root);
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].system, IncumbentSystem::Tolgee);
    }

    /// Real fixture: `strawberrymusicplayer/strawberry`'s crowdin.yml — a
    /// single-file mapping (Qt .ts format), the simplest real shape.
    #[test]
    fn test_detect_crowdin_real_world_shape_strawberry_single_file() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        fs::write(
            root.join("crowdin.yml"),
            "files:\n  - source: /src/translations/strawberry_en_US.ts\n    translation: /src/translations/strawberry_%locale_with_underscore%.ts\n",
        )
        .unwrap();

        let imports = detect_incumbent_systems(root);
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].include_patterns, vec!["/src/translations/strawberry_en_US.ts"]);
    }

    /// Real fixture: `microsoft/samples-for-ai`'s crowdin.yml — the most
    /// structurally complex real shape found: three separate `files:`
    /// entries (.md/.png/.gif), each with a nested `ignore:` list and a
    /// `languages_mapping:` object interleaved between `source:` lines.
    /// Confirms the line-scanner doesn't get confused by the nested `ignore:`
    /// list items (which also start with `- `) and still extracts all three
    /// real source patterns correctly.
    #[test]
    fn test_detect_crowdin_real_world_shape_samples_for_ai_multi_file_complex() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        let crowdin_yml = r#"files:
  - source: /**/*.md
    ignore:
      - /**/Progressive growing of GANs*.md
      - /**/ThirdPartyNotice.md
      - /**/LICENSE.md
    translation: /%locale_with_underscore%/%original_path%/%original_file_name%
    languages_mapping:
      locale_with_underscore:
        zh-CN: zh-hans
  - source: /**/*.png
    translation: /%locale_with_underscore%/%original_path%/%original_file_name%
  - source: /**/*.gif
    translation: /%three_letters_code%/%original_path%/%original_file_name%
"#;
        fs::write(root.join("crowdin.yml"), crowdin_yml).unwrap();

        let imports = detect_incumbent_systems(root);
        assert_eq!(imports.len(), 1);
        assert_eq!(
            imports[0].include_patterns,
            vec!["/**/*.md", "/**/*.png", "/**/*.gif"],
            "must extract all 3 real source patterns despite the nested ignore: list and languages_mapping: block in between"
        );
    }

    /// Real fixture: verbatim `crowdin.yml` from `DS-Homebrew/TWiLightMenu`
    /// (a real, live Crowdin user). Regression guard for the bug where the
    /// parser only looked for `source_language:`/`languages:` keys that
    /// don't exist in this file, silently extracting nothing.
    #[test]
    fn test_detect_crowdin_real_world_shape_twilightmenu() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        let crowdin_yml = r#"project_id: 407378
api_token_env: CROWDIN_TOKEN
preserve_hierarchy: true

files:
  - source: /manual/nitrofiles/languages/en/*
    translation: /manual/nitrofiles/languages/%two_letters_code%/%original_file_name%
  - source: /quickmenu/nitrofiles/languages/en/*
    translation: /quickmenu/nitrofiles/languages/%two_letters_code%/%original_file_name%
  - source: /settings/nitrofiles/languages/en/*
    translation: /settings/nitrofiles/languages/%two_letters_code%/%original_file_name%
"#;
        fs::write(root.join("crowdin.yml"), crowdin_yml).unwrap();

        let imports = detect_incumbent_systems(root);
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].system, IncumbentSystem::Crowdin);
        assert_eq!(
            imports[0].include_patterns,
            vec![
                "/manual/nitrofiles/languages/en/*",
                "/quickmenu/nitrofiles/languages/en/*",
                "/settings/nitrofiles/languages/en/*",
            ],
            "must extract the real files[].source patterns since no source_language/languages keys exist"
        );
    }

    /// Second independent real fixture — `mendhak/gpslogger`'s crowdin.yml,
    /// a single-file mapping rather than TWiLightMenu's multi-file list.
    #[test]
    fn test_detect_crowdin_real_world_shape_gpslogger() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        fs::write(
            root.join("crowdin.yml"),
            "files:\n  - source: /gpslogger/src/main/res/values/strings.xml\n    translation: /gpslogger/src/main/res/values-%two_letters_code%/strings.xml\n    translate_attributes: 0\n",
        )
        .unwrap();

        let imports = detect_incumbent_systems(root);
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].system, IncumbentSystem::Crowdin);
        assert_eq!(imports[0].include_patterns, vec!["/gpslogger/src/main/res/values/strings.xml"]);
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

    /// Real fixture: verbatim `project.inlang/settings.json` from
    /// `VERT-sh/VERT`, using the current `baseLocale`/`locales` schema
    /// (`@inlang/plugin-message-format`) rather than AppFlowy's legacy
    /// `sourceLanguageTag`/`languageTags` (`@inlang/plugin-i18next`).
    /// Regression guard for the bug where only the legacy field names were
    /// read, so this real, currently-active project's locale data was
    /// silently dropped even though the file was correctly detected.
    #[test]
    fn test_detect_inlang_real_world_shape_vert_message_format_schema() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        let inlang_dir = root.join("project.inlang");
        fs::create_dir_all(&inlang_dir).unwrap();
        let settings = r#"
        {
          "$schema": "https://inlang.com/schema/project-settings",
          "baseLocale": "en",
          "locales": ["en", "es", "fr", "de", "it", "ba", "hr", "tr", "ja", "ko", "el", "id", "zh-Hans", "zh-Hant", "pt-BR"],
          "modules": ["https://cdn.jsdelivr.net/npm/@inlang/plugin-message-format@4/dist/index.js"],
          "plugin.inlang.messageFormat": { "pathPattern": "./messages/{locale}.json" }
        }
        "#;
        fs::write(inlang_dir.join("settings.json"), settings).unwrap();

        let imports = detect_incumbent_systems(root);
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].system, IncumbentSystem::Inlang);
        assert_eq!(imports[0].source_locale.as_deref(), Some("en"), "must read baseLocale when sourceLanguageTag is absent");
        assert_eq!(imports[0].target_locales.len(), 14, "15 total locales minus the source itself");
        assert!(imports[0].target_locales.contains(&"zh-Hans".to_string()));
    }

    /// Second independent real project on the same current schema —
    /// `Splode/pomotroid` — confirming this isn't a one-off.
    #[test]
    fn test_detect_inlang_real_world_shape_pomotroid_message_format_schema() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        let inlang_dir = root.join("project.inlang");
        fs::create_dir_all(&inlang_dir).unwrap();
        fs::write(
            inlang_dir.join("settings.json"),
            r#"{"$schema": "https://inlang.com/schema/project-settings", "baseLocale": "en", "locales": ["en", "es", "fr", "de", "ja", "zh", "pt", "tr"], "modules": [], "plugin.inlang.messageFormat": {"pathPattern": "./src/messages/{locale}.json"}}"#,
        )
        .unwrap();

        let imports = detect_incumbent_systems(root);
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].source_locale.as_deref(), Some("en"));
        assert_eq!(imports[0].target_locales.len(), 7);
    }

    /// Real fixture: verbatim `project.inlang/settings.json` from
    /// `AppFlowy-IO/AppFlowy` (76.8k stars, a real, live inlang user).
    /// Positive-confirmation regression guard — unlike the Lingo.dev/Tolgee/
    /// Crowdin bugs, this path was already correct, but a real 30-language,
    /// region-tagged (`pt-BR`/`zh-CN`/`fr-CA` style) fixture is worth locking
    /// in so a future change can't silently break it.
    #[test]
    fn test_detect_inlang_real_world_shape_appflowy() {
        let tmp = tempdir().unwrap();
        let root = tmp.path();
        let inlang_dir = root.join("project.inlang");
        fs::create_dir_all(&inlang_dir).unwrap();
        let settings = r#"
        {
          "$schema": "https://inlang.com/schema/project-settings",
          "sourceLanguageTag": "en-US",
          "languageTags": [
            "en-US", "en-GB", "ar-SA", "ca-ES", "ckb-KU", "cs-CZ", "de-DE", "es-VE",
            "eu-ES", "fa", "fr-CA", "fr-FR", "ga-IE", "he", "hu-HU", "id-ID", "it-IT",
            "ja-JP", "ko-KR", "pl-PL", "pt-BR", "pt-PT", "ru-RU", "sv-SE", "th-TH",
            "tr-TR", "uk-UA", "vi", "vi-VN", "zh-CN", "zh-TW"
          ],
          "modules": ["https://cdn.jsdelivr.net/npm/@inlang/plugin-i18next@4/dist/index.js"],
          "plugin.inlang.i18next": {
            "pathPattern": "./frontend/resources/translations/{languageTag}.json"
          }
        }
        "#;
        fs::write(inlang_dir.join("settings.json"), settings).unwrap();

        let imports = detect_incumbent_systems(root);
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].system, IncumbentSystem::Inlang);
        assert_eq!(imports[0].source_locale.as_deref(), Some("en-US"));
        assert_eq!(imports[0].target_locales.len(), 30, "31 total tags minus the source itself");
        assert!(imports[0].target_locales.contains(&"zh-CN".to_string()));
        assert!(!imports[0].target_locales.contains(&"en-US".to_string()), "source tag must be filtered out of targets");
    }
}
