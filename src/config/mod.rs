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
    pub output_format: String, // "json", "po", "fluent", "inline"
    pub locales_dir: String,
}

impl Default for LingoConfig {
    fn default() -> Self {
        Self {
            source_locale: "en".to_string(),
            target_locales: vec!["es".to_string(), "fr".to_string(), "de".to_string()],
            include: vec![
                "src/**/*.{tsx,jsx,vue,svelte,html,md,json}".to_string(),
            ],
            exclude: vec![
                "node_modules/**".to_string(),
                "dist/**".to_string(),
                ".next/**".to_string(),
                "target/**".to_string(),
            ],
            framework: None,
            provider: "offline".to_string(),
            output_format: "json".to_string(),
            locales_dir: "locales".to_string(),
        }
    }
}

impl LingoConfig {
    /// Load config from `lingo.json` or default.
    pub fn load_or_default<P: AsRef<Path>>(dir: P) -> Self {
        let json_path = dir.as_ref().join("lingo.json");
        if json_path.exists() {
            if let Ok(content) = fs::read_to_string(&json_path) {
                if let Ok(cfg) = serde_json::from_str::<Self>(&content) {
                    return cfg;
                }
            }
        }
        Self::default()
    }

    /// Save config to `lingo.json`.
    pub fn save_to_file<P: AsRef<Path>>(&self, dir: P) -> Result<PathBuf, std::io::Error> {
        let path = dir.as_ref().join("lingo.json");
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        fs::write(&path, content)?;
        Ok(path)
    }

    /// Auto-detect framework from repository dependencies or files.
    pub fn detect_framework<P: AsRef<Path>>(dir: P) -> Option<TargetFramework> {
        let root = dir.as_ref();
        let pkg_json_path = root.join("package.json");

        if pkg_json_path.exists() {
            if let Ok(content) = fs::read_to_string(pkg_json_path) {
                let lower = content.to_ascii_lowercase();
                if lower.contains("\"next\"") || lower.contains("\"react\"") {
                    return Some(TargetFramework::React);
                } else if lower.contains("\"vue\"") || lower.contains("\"nuxt\"") {
                    return Some(TargetFramework::Vue);
                } else if lower.contains("\"svelte\"") || lower.contains("\"@sveltejs/kit\"") {
                    return Some(TargetFramework::Svelte);
                } else if lower.contains("\"solid-js\"") {
                    return Some(TargetFramework::Solid);
                }
            }
        }

        // Fallback: check file existence
        if root.join("svelte.config.js").exists() {
            return Some(TargetFramework::Svelte);
        }
        if root.join("nuxt.config.ts").exists() || root.join("nuxt.config.js").exists() {
            return Some(TargetFramework::Vue);
        }
        if root.join("next.config.js").exists() || root.join("next.config.mjs").exists() {
            return Some(TargetFramework::React);
        }

        None
    }
}
