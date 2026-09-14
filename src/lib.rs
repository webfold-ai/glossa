//! # Ptyktos Lingo — Universal Cross-Framework Localization on Apeiron Basis
//!
//! Recreating lingo.dev in pure Rust and orders of magnitude faster,
//! designed in the cross-framework spirit of Mitosis.
//!
//! ## Mathematical & Architectural Basis
//! Grounded in the Sanguine/Fold theorem `adapt = fromTerm ∘ universalLift`
//! (Formalized in `Substrate.Algebra.Translation.UniversalAdapter.lean`).
//!
//! 1. **The Ingest (`universalLift`)**: High-speed, streaming zero-copy extractors for
//!    React/JSX, Vue SFC, Svelte, Solid, HTML/Templates (Askama, Tera, Jinja, Handlebars),
//!    Markdown/MDX, and translation files (JSON, PO, Fluent).
//! 2. **The Invariant Ground (`ApeironTerm`)**: Discretized intermediate representation
//!    where framework syntax differences are quotiented out. Coordinated via
//!    Fibonacci/hashtrinity hashing.
//! 3. **The Out-Leg (`fromTerm`)**: Compilers for React, Vue, Svelte, Solid, and
//!    Zero-Runtime Static Evaluation.
//! 4. **The Null-Gate (`gate_kills_on_the_cone`)**: Boundary verification ensuring
//!    slot and tag conservation without AI hallucinations or security leaks.
//! 5. **Monoidal Lockfile (`lingo.lock`)**: Conflict-free commutative merging (`Table::fuse`)
//!    and sub-millisecond delta computation.

pub mod cli;
pub mod config;
pub mod emitters;
pub mod engine;
pub mod ir;
pub mod lockfile;
pub mod mcp;
pub mod parsers;

pub use config::LingoConfig;
pub use emitters::{emit_trans_call, TargetFramework};
pub use engine::{gate::ApeironNullGate, TranslationEngine, TranslationProvider};
pub use ir::{ApeironCatalog, ApeironCoord, ApeironTerm, ExtractedItem};
pub use lockfile::LingoLock;
pub use parsers::{universal_lift, SourceFormat};
