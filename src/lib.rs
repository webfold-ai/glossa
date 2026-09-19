//! # Glossa — High-Performance Sovereign Multi-Framework Localization Engine
//!
//! Ultra-fast Rust localization compiler, CI/CD verification runner, and translation engine.
//! Designed as an ultra-fast, cost-effective drop-in replacement for Lingo.dev, Tolgee, Inlang, and Crowdin.
//!
//! ## Core Architecture
//!
//! 1. **Universal Extraction (`universal_lift`)**: High-speed, streaming zero-copy extractors for
//!    React/JSX, Vue SFC, Svelte, Solid, HTML/Templates, Markdown/MDX, Mobile (iOS/Android/Flutter),
//!    Backend (Python/Go/Ruby/PHP), and translation catalogs (JSON, YAML, PO, Fluent, XLIFF).
//! 2. **Canonical IR (`CanonicalTerm`)**: Unified intermediate representation quotienting out framework
//!    syntax differences. Coordinated via deterministic content-addressed 64-bit coordinates (`ContentCoord`).
//! 3. **Framework Emitters (`from_term`)**: Optimized emitters for React, Vue, Svelte, Solid, and
//!    Zero-Runtime Static Evaluation.
//! 4. **Integrity Gate (`IntegrityGate`)**: Mathematical structural verification ensuring
//!    slot conservation, tag parity, and prompt-injection neutralization.
//! 5. **Monoidal Lockfile (`glossa.lock`)**: Conflict-free commutative merging (`Table::fuse`)
//!    and sub-millisecond delta computation guaranteeing 0 Git merge conflicts.

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
