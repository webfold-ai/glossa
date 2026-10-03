//! The Intermediate Representation (IR) module.

pub mod coord;
pub mod term;

pub use coord::{ApeironCoord, ContentCoord, PHI64};
pub use term::{
    normalize_whitespace, ApeironCatalog, ApeironTerm, CanonicalTerm, ExtractedItem,
    PluralCategory, SourceLocation,
};
