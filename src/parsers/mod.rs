//! The Universal Ingest Module: `universalLift`.
//!
//! Theorem: `adapt = fromTerm ∘ universalLift` (Sanguine/UniversalAdapter).
//! Dispatches to framework-specific streaming extractors (JSX, Vue, Svelte, HTML, Markdown, JSON, PO, Fluent)
//! to produce canonical ApeironTerm units.

pub mod html;
pub mod jsx;
pub mod markdown;
pub mod resource;
pub mod svelte;
pub mod vue;

use crate::ir::term::{ApeironCatalog, ExtractedItem};
use std::path::Path;
use walkdir::WalkDir;

/// Supported source file formats.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SourceFormat {
    Jsx,
    Vue,
    Svelte,
    Html,
    Markdown,
    Json,
    Po,
    Fluent,
}

impl SourceFormat {
    pub fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        match ext.as_str() {
            "jsx" | "tsx" | "js" | "ts" => Some(Self::Jsx),
            "vue" => Some(Self::Vue),
            "svelte" => Some(Self::Svelte),
            "html" | "htm" | "tera" | "jinja" | "jinja2" | "hbs" => Some(Self::Html),
            "md" | "mdx" | "markdown" => Some(Self::Markdown),
            "json" => Some(Self::Json),
            "po" | "pot" => Some(Self::Po),
            "ftl" => Some(Self::Fluent),
            _ => None,
        }
    }
}

/// Extract translatable items from source text using the specified format.
pub fn universal_lift(source: &str, format: SourceFormat, file_path: &str) -> Vec<ExtractedItem> {
    match format {
        SourceFormat::Jsx => jsx::extract_jsx(source, file_path),
        SourceFormat::Vue => vue::extract_vue(source, file_path),
        SourceFormat::Svelte => svelte::extract_svelte(source, file_path),
        SourceFormat::Html => html::extract_html(source, file_path),
        SourceFormat::Markdown => markdown::extract_markdown(source, file_path),
        SourceFormat::Json => resource::extract_json(source, file_path),
        SourceFormat::Po => resource::extract_po(source, file_path),
        SourceFormat::Fluent => resource::extract_fluent(source, file_path),
    }
}

/// Recursively scans a directory and extracts translatable items into an ApeironCatalog.
pub fn extract_directory(root_dir: &Path, ignore_patterns: &[&str]) -> Result<ApeironCatalog, std::io::Error> {
    let mut catalog = ApeironCatalog::new();

    for entry in WalkDir::new(root_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }

        let path_str = path.to_string_lossy();

        // Check ignore patterns (node_modules, target, .git, dist, etc.)
        if is_ignored(&path_str, ignore_patterns) {
            continue;
        }

        if let Some(format) = SourceFormat::from_path(path) {
            if let Ok(content) = std::fs::read_to_string(path) {
                let items = universal_lift(&content, format, &path_str);
                for item in items {
                    catalog.insert(item);
                }
            }
        }
    }

    Ok(catalog)
}

fn is_ignored(path: &str, patterns: &[&str]) -> bool {
    let default_ignores = &[
        "/target/",
        "/.git/",
        "/node_modules/",
        "/.next/",
        "/dist/",
        "/build/",
        "/cache/",
    ];
    for ign in default_ignores {
        if path.contains(ign) {
            return true;
        }
    }
    for pat in patterns {
        if path.contains(pat) {
            return true;
        }
    }
    false
}
