//! Project Configuration (`lingo.json` / `lingo.toml`).

use crate::emitters::TargetFramework;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LingoConfig {
    pub source_locale: String,
    pub target_locales: Vec<String>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub framework: Option<String>,
    pub provider: String,
    pub output_format: String, // "json", "yaml", "po", "fluent", "android", "ios", "arb", "properties", "xliff", "csv"
    pub locales_dir: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub glossary: Option<std::collections::BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brand_tone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buckets: Option<Vec<String>>,
}

impl Default for LingoConfig {
    fn default() -> Self {
        Self {
            source_locale: "en".to_string(),
            target_locales: vec!["es".to_string(), "fr".to_string(), "de".to_string()],
            include: vec![
                "src/**/*.{tsx,jsx,vue,svelte,html,md,json,yaml,yml,po,ftl,arb,properties,swift,dart,py,go,rb,php}".to_string(),
            ],
            exclude: vec![
                "node_modules/**".to_string(),
                "dist/**".to_string(),
                ".next/**".to_string(),
                "target/**".to_string(),
                ".git/**".to_string(),
            ],
            framework: None,
            provider: "offline".to_string(),
            output_format: "json".to_string(),
            locales_dir: "locales".to_string(),
            glossary: None,
            brand_tone: None,
            buckets: None,
        }
    }
}

pub type GlossaConfig = LingoConfig;

impl LingoConfig {
    /// Load config from `glossa.json` (or fallback `lingo.json`) or default.
    pub fn load_or_default<P: AsRef<Path>>(dir: P) -> Self {
        let glossa_path = dir.as_ref().join("glossa.json");
        if glossa_path.exists() {
            if let Ok(content) = fs::read_to_string(&glossa_path) {
                if let Ok(cfg) = serde_json::from_str::<Self>(&content) {
                    return cfg;
                }
            }
        }
        let json_path = dir.as_ref().join("lingo.json");
        if json_path.exists() {
            if let Ok(content) = fs::read_to_string(&json_path) {
                if let Ok(cfg) = serde_json::from_str::<Self>(&content) {
                    return cfg;
                }
            }
        }
        // Lingo.dev's current real config is `i18n.json`, whose schema
        // (`locale: {source, targets}`, object-shaped `provider`, `buckets`)
        // does not match `LingoConfig`'s own field layout, so it can't be
        // deserialized directly into `Self` the way `lingo.json` is above.
        // Route it through the importer's real-schema parser instead.
        if dir.as_ref().join("i18n.json").exists() {
            let mut cfg = Self::default();
            for import in crate::importer::detect_incumbent_systems(dir.as_ref()) {
                if import.system == crate::importer::IncumbentSystem::LingoDev {
                    crate::importer::import_to_glossa_config(&import, &mut cfg);
                    return cfg;
                }
            }
        }
        Self::default()
    }

    /// Save config to `glossa.json` (or `lingo.json` if present).
    pub fn save_to_file<P: AsRef<Path>>(&self, dir: P) -> Result<PathBuf, std::io::Error> {
        let path = if dir.as_ref().join("lingo.json").exists() {
            dir.as_ref().join("lingo.json")
        } else {
            dir.as_ref().join("glossa.json")
        };
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        fs::write(&path, content)?;
        Ok(path)
    }

    /// Auto-detect framework or language from repository dependencies or files.
    pub fn detect_framework_name<P: AsRef<Path>>(dir: P) -> Option<String> {
        let root = dir.as_ref();
        let pkg_json_path = root.join("package.json");

        if pkg_json_path.exists() {
            if let Ok(content) = fs::read_to_string(pkg_json_path) {
                let lower = content.to_ascii_lowercase();
                if lower.contains("\"next\"") {
                    return Some("nextjs".to_string());
                } else if lower.contains("\"react\"") {
                    return Some("react".to_string());
                } else if lower.contains("\"nuxt\"") {
                    return Some("nuxt".to_string());
                } else if lower.contains("\"vue\"") {
                    return Some("vue".to_string());
                } else if lower.contains("\"@sveltejs/kit\"") || lower.contains("\"svelte\"") {
                    return Some("svelte".to_string());
                } else if lower.contains("\"solid-js\"") {
                    return Some("solid".to_string());
                } else if lower.contains("\"@angular/core\"") {
                    return Some("angular".to_string());
                } else if lower.contains("\"astro\"") {
                    return Some("astro".to_string());
                }
            }
        }

        if root.join("pubspec.yaml").exists() {
            return Some("flutter".to_string());
        }
        if root.join("Package.swift").exists() {
            return Some("swift".to_string());
        }
        if root.join("build.gradle").exists() || root.join("build.gradle.kts").exists() {
            return Some("android".to_string());
        }
        if root.join("requirements.txt").exists() || root.join("pyproject.toml").exists() {
            return Some("python".to_string());
        }
        if root.join("go.mod").exists() {
            return Some("go".to_string());
        }
        if root.join("Gemfile").exists() {
            return Some("rails".to_string());
        }
        if root.join("composer.json").exists() {
            return Some("laravel".to_string());
        }
        if root.join("Cargo.toml").exists() {
            return Some("rust".to_string());
        }

        None
    }

    /// Auto-detect framework into legacy TargetFramework enum.
    pub fn detect_framework<P: AsRef<Path>>(dir: P) -> Option<TargetFramework> {
        match Self::detect_framework_name(dir)?.as_str() {
            "react" | "nextjs" => Some(TargetFramework::React),
            "vue" | "nuxt" => Some(TargetFramework::Vue),
            "svelte" => Some(TargetFramework::Svelte),
            "solid" => Some(TargetFramework::Solid),
            _ => None,
        }
    }
}
