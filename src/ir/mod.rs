//! The Apeiron IR module.

pub mod coord;
pub mod term;

pub use coord::{ApeironCoord, PHI64};
pub use term::{
    normalize_whitespace, ApeironCatalog, ApeironTerm, ExtractedItem, PluralCategory,
    SourceLocation,
};
