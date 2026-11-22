//! # Glossa — multi-framework localization engine
//!
//! A Rust localization tool: it extracts translatable strings from source code and catalogs,
//! stores them in a lockfile, translates missing entries through a configurable provider,
//! checks the result with an integrity gate, and writes locale catalogs.
//!
//! ## Core Architecture
//!
//! 1. **Extraction (`universal_lift`)**: extractors for React/JSX, Vue SFC, Svelte,
//!    HTML templates, Markdown/MDX, translation-call patterns in Python/Go/Ruby/PHP/Swift/Dart,
//!    Angular and Astro sources, and catalogs (JSON, YAML, PO, Fluent, XLIFF, Android XML, iOS
//!    `.strings`, Flutter ARB, Properties, CSV).
//! 2. **Canonical IR (`CanonicalTerm`)**: Intermediate representation shared by all extractors.
//!    Each unit gets a deterministic content-addressed 64-bit coordinate (`ContentCoord`).
//! 3. **Framework Emitters (`from_term`)**: Emitters for React, Vue, Svelte, Solid, and static evaluation.
//! 4. **Integrity Gate (`IntegrityGate`)**: Checks that a translation keeps the source slots
//!    (`{name}`), keeps numbered tags balanced, and contains no script or dangerous HTML.
//! 5. **Monoidal Lockfile (`glossa.lock`)**: Lockfile entries merge with `Table::fuse`, which
//!    is order-independent, and the lockfile computes which translations are missing.

pub mod cli;
pub mod codegen;
pub mod config;
pub mod emitters;
pub mod engine;
pub mod importer;
pub mod ir;
pub mod lockfile;
pub mod mcp;
pub mod parsers;
pub mod visual;

pub use config::{GlossaConfig, LingoConfig};
pub use emitters::{emit_trans_call, TargetFramework};
pub use engine::{
    gate::ApeironNullGate,
    gate::IntegrityGate,
    locale_map::{
        dominant_locale, locale_map, parts_needing_lang_attr, primary_locale, LocaleRegion,
        LocaleStatus,
    },
    TranslationEngine, TranslationProvider,
};
pub use ir::{
    ApeironCatalog, ApeironCoord, ApeironTerm, CanonicalTerm, ContentCoord, ExtractedItem,
};
pub use lockfile::LingoLock;
pub use parsers::{universal_lift, SourceFormat};
