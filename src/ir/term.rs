//! Universal Intermediate Representation (IR) for Translatable Units.
//!
//! Multi-framework surfaces (React JSX, Vue template, Svelte, Jinja, Markdown, Fluent)
//! are lifted into `CanonicalTerm` via `universal_lift`, quotienting out syntax noise.
//! Translation is performed on the canonical term, and target code is generated
//! via `from_term`.

use crate::ir::coord::ApeironCoord;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// ICU Plural categories for localization.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PluralCategory {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

impl PluralCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Zero => "zero",
            Self::One => "one",
            Self::Two => "two",
            Self::Few => "few",
            Self::Many => "many",
            Self::Other => "other",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "zero" | "=0" => Some(Self::Zero),
            "one" | "=1" => Some(Self::One),
            "two" | "=2" => Some(Self::Two),
            "few" => Some(Self::Few),
            "many" => Some(Self::Many),
            "other" => Some(Self::Other),
            _ => None,
        }
    }
}

/// A node in the Apeiron universal translation term.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApeironTerm {
    /// Pure text segment.
    Text(String),
    /// Dynamic interpolation variable / slot (e.g. `{userName}` or `{count + 1}`).
    Slot {
        id: usize,
        name: String,
        raw_expr: Option<String>,
    },
    /// Inline markup container (e.g. `<b>...</b>`, `<Link href="...">...</Link>`).
    Tag {
        id: usize,
        tag: String,
        attrs: Vec<(String, String)>,
        children: Vec<ApeironTerm>,
    },
    /// ICU-style pluralization branch.
    Plural {
        slot_name: String,
        cases: Vec<(PluralCategory, Vec<ApeironTerm>)>,
    },
    /// ICU-style select branch (e.g. gender or status enum).
    Select {
        slot_name: String,
        cases: Vec<(String, Vec<ApeironTerm>)>,
    },
}

/// Alias for CanonicalTerm representing universal translation intermediate representation node.
pub type CanonicalTerm = ApeironTerm;

impl ApeironTerm {
    /// Creates a flat sequence of terms into a canonical representation.
    pub fn normalize_sequence(nodes: Vec<ApeironTerm>) -> Vec<ApeironTerm> {
        let mut out = Vec::new();
        let mut current_text = String::new();

        for node in nodes {
            match node {
                ApeironTerm::Text(t) => {
                    current_text.push_str(&t);
                }
                other => {
                    if !current_text.is_empty() {
                        let norm = normalize_whitespace(&current_text);
                        if !norm.is_empty() {
                            out.push(ApeironTerm::Text(norm));
                        }
                        current_text.clear();
                    }
                    out.push(other);
                }
            }
        }
        if !current_text.is_empty() {
            let norm = normalize_whitespace(&current_text);
            if !norm.is_empty() {
                out.push(ApeironTerm::Text(norm));
            }
        }
        out
    }

    /// Render a list of terms to canonical representation for hashing and LLM prompting.
    pub fn to_canonical_string(terms: &[ApeironTerm]) -> String {
        let mut out = String::new();
        for term in terms {
            match term {
                ApeironTerm::Text(s) => out.push_str(s),
                ApeironTerm::Slot { name, .. } => {
                    out.push('{');
                    out.push_str(name);
                    out.push('}');
                }
                ApeironTerm::Tag { id, children, .. } => {
                    out.push_str(&format!("<{}>", id));
                    out.push_str(&Self::to_canonical_string(children));
                    out.push_str(&format!("</{}>", id));
                }
                ApeironTerm::Plural { slot_name, cases } => {
                    out.push_str(&format!("{{{slot_name}, plural, "));
                    for (cat, branch) in cases {
                        out.push_str(&format!("{} {{{}}} ", cat.as_str(), Self::to_canonical_string(branch)));
                    }
                    out.push('}');
                }
                ApeironTerm::Select { slot_name, cases } => {
                    out.push_str(&format!("{{{slot_name}, select, "));
                    for (val, branch) in cases {
                        out.push_str(&format!("{val} {{{}}} ", Self::to_canonical_string(branch)));
                    }
                    out.push('}');
                }
            }
        }
        out
    }

    /// Computes the invariant Apeiron coordinate for a list of terms.
    pub fn compute_coord(terms: &[ApeironTerm]) -> ApeironCoord {
        let canon = Self::to_canonical_string(terms);
        ApeironCoord::from_str_canonical(&canon)
    }

    /// Extracts all slot names present in the term tree.
    pub fn collect_slots(terms: &[ApeironTerm]) -> Vec<String> {
        let mut slots = Vec::new();
        for t in terms {
            match t {
                ApeironTerm::Slot { name, .. } => slots.push(name.clone()),
                ApeironTerm::Tag { children, .. } => slots.extend(Self::collect_slots(children)),
                ApeironTerm::Plural { slot_name, cases } => {
                    slots.push(slot_name.clone());
                    for (_, branch) in cases {
                        slots.extend(Self::collect_slots(branch));
                    }
                }
                ApeironTerm::Select { slot_name, cases } => {
                    slots.push(slot_name.clone());
                    for (_, branch) in cases {
                        slots.extend(Self::collect_slots(branch));
                    }
                }
                ApeironTerm::Text(_) => {}
            }
        }
        slots.sort();
        slots.dedup();
        slots
    }
}

/// Normalizes runs of whitespace into a single ASCII space.
pub fn normalize_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_was_ws = false;

    for ch in s.chars() {
        if ch.is_whitespace() {
            if !last_was_ws {
                out.push(' ');
                last_was_ws = true;
            }
        } else {
            out.push(ch);
            last_was_ws = false;
        }
    }
    out
}

/// Location in source code where a translatable term was discovered.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file_path: String,
    pub line: usize,
    pub column: usize,
}

/// A translatable item extracted from a source codebase.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedItem {
    pub coord: ApeironCoord,
    pub key: Option<String>,
    pub terms: Vec<ApeironTerm>,
    pub canonical_text: String,
    pub context: Option<String>,
    pub locations: Vec<SourceLocation>,
}

/// A project-wide catalog of extracted terms.
/// Forms a commutative monoid under union: `Table::fuse`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApeironCatalog {
    pub items: BTreeMap<ApeironCoord, ExtractedItem>,
}

impl ApeironCatalog {
    pub fn new() -> Self {
        Self {
            items: BTreeMap::new(),
        }
    }

    /// Add an item into the catalog, merging occurrences if already present.
    pub fn insert(&mut self, mut item: ExtractedItem) {
        if let Some(existing) = self.items.get_mut(&item.coord) {
            existing.locations.append(&mut item.locations);
            existing.locations.sort_by(|a, b| (&a.file_path, a.line).cmp(&(&b.file_path, b.line)));
            existing.locations.dedup();
            if existing.key.is_none() && item.key.is_some() {
                existing.key = item.key;
            }
            if existing.context.is_none() && item.context.is_some() {
                existing.context = item.context;
            }
        } else {
            self.items.insert(item.coord, item);
        }
    }

    /// Commutative monoidal merge of two catalogs. Order-free and shard-mergeable.
    pub fn merge(&mut self, other: ApeironCatalog) {
        for (_, item) in other.items {
            self.insert(item);
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl fmt::Display for ApeironTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", ApeironTerm::to_canonical_string(&[self.clone()]))
    }
}
