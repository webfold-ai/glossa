//! Locale Cross-Reference API — WCAG `lang`-attribute ground truth.
//!
//! Glossa is the only component in the stack that actually knows the *true* language
//! of a piece of content (translation catalogs are keyed by locale — that's the job).
//! This module exposes that knowledge in a form a separate crate (e.g. `ptyktos-qa`,
//! a static WCAG accessibility auditor for rendered HTML) can cross-reference against
//! a page's declared `<html lang="...">` / `lang="..."` attributes, to check:
//!
//! - **WCAG SC 3.1.1 Language of Page (A)**: does the page's declared `lang` actually
//!   match the dominant language of its content? See [`dominant_locale`].
//! - **WCAG SC 3.1.2 Language of Parts (AA)**: are there passages whose *actual*
//!   language differs from the page's declared language and therefore need their own
//!   `lang="..."` attribute? See [`parts_needing_lang_attr`].
//!
//! ## What Glossa's data model actually supports
//!
//! Two structures carry the relevant information, and neither alone is sufficient:
//!
//! - [`ApeironCatalog`](crate::ir::ApeironCatalog) is the result of extraction
//!   (`universal_lift`). It is **locale-agnostic**: every [`ExtractedItem`] in a
//!   catalog is the literal text as written in the source files, in whatever single
//!   language the project's source is authored in. There is no per-item locale field
//!   on `ExtractedItem` or `ApeironCoord` — extraction does not know about locales at
//!   all, only content and source position.
//! - [`LingoLock`](crate::lockfile::LingoLock) (`glossa.lock` / `lingo.lock`) is where
//!   locale information actually lives, per fragment: `LockEntry.locales` is a
//!   `BTreeMap<String, LocaleEntry>` keyed by locale code, populated only for
//!   fragments that have been translated into that locale.
//!
//! So Glossa's locale tracking is **per-catalog for the source language, per-fragment
//! for translated locales**: the whole catalog is written in one source locale (see
//! [`primary_locale`]), and on top of that, individual fragments may or may not have
//! a translation into any given target locale (see [`locale_map`]). A fragment with
//! no translation into the target locale a page was built for is rendered as
//! source-locale fallback text — this is *exactly* the WCAG SC 3.1.2 "parts" case:
//! a `fr` page with an untranslated `en` string still embedded in it.
//!
//! ## Location format, and what a consumer must do with it
//!
//! The only location Glossa tracks is [`SourceLocation`] — `file_path` / `line` /
//! `column` in the *original source file* (the `.tsx`, `.vue`, `.svelte`, etc. that was
//! parsed). Glossa does **not** track CSS selectors, XPath, DOM node identity, or any
//! rendered-output position — no such correlation exists anywhere in this codebase
//! (extraction, emitters, and the visual-editor server all stop at source position or
//! literal text matching).
//!
//! A consumer (such as ptyktos-qa, which audits *rendered* HTML) therefore cannot use
//! `SourceLocation` as a DOM selector directly. To turn a [`LocaleRegion`] into a DOM
//! `lang`-attribute check, a caller needs to bridge source position to rendered output
//! itself, e.g. by:
//!
//! 1. **Text-content matching** (works today, no glossa changes needed): match
//!    [`LocaleRegion::rendered_text`] against the rendered DOM's text nodes. This is
//!    the same strategy Glossa's own in-context visual editor uses
//!    (`src/visual/mod.rs`, `innerText` matching) — imprecise for repeated/short
//!    strings, exact for longer sentences.
//! 2. **Build-time source maps**, if the consuming framework's toolchain emits them,
//!    to map `file_path:line:column` to a rendered DOM node.
//! 3. **A future glossa enhancement** (not implemented here): emitters could inject a
//!    `data-glossa-coord="<hex coord>"` attribute onto the emitted element, giving an
//!    exact, unambiguous DOM selector (`[data-glossa-coord="..."]`) instead of relying
//!    on text matching. This module's [`ApeironCoord`] values are already stable and
//!    ready to be used as that attribute's value if/when such an emitter change lands;
//!    no changes to this module would be required.
//!
//! Option 1 is the only one usable purely from what this module returns today — it is
//! documented here precisely so a consumer doesn't have to guess.

use crate::ir::coord::ApeironCoord;
use crate::ir::term::{ApeironCatalog, SourceLocation};
use crate::lockfile::LingoLock;
use serde::{Deserialize, Serialize};

/// Whether a fragment's rendered text in a given target-locale build is an actual
/// translation into that locale, or a fallback to the catalog's source-locale text
/// because no translation exists yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocaleStatus {
    /// A translation into the requested target locale exists in the lockfile and is
    /// what will render for this fragment.
    Translated,
    /// No translation into the requested target locale exists yet; the source-locale
    /// text is rendered as fallback. If the page declares a different `lang`, this is
    /// a WCAG SC 3.1.2 candidate.
    Fallback,
}

/// One content fragment's effective locale within a specific target-locale build, as
/// known to Glossa.
///
/// Produced by [`locale_map`] by cross-referencing an [`ApeironCatalog`] (source-locale
/// extraction) against a [`LingoLock`] (per-fragment translation records) for one
/// requested target locale.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocaleRegion {
    /// Content-addressed coordinate identifying this translatable fragment. Stable
    /// across framework rewrites (React -> Vue, etc.) — see [`ApeironCoord`].
    pub coord: ApeironCoord,
    /// Where this fragment originates in source (file/line/column). See the module
    /// docs for what a consumer must do to map this to a rendered DOM element — this
    /// is a source position, not a DOM selector.
    pub locations: Vec<SourceLocation>,
    /// Developer-assigned translation key, if the source used one (e.g. `common.welcome`).
    pub key: Option<String>,
    /// ISO/BCP-47 locale code Glossa believes this fragment will actually render as
    /// (e.g. `"fr"`, `"en"`, `"fr-CA"`) for the requested target-locale build. Equal to
    /// the requested target locale when [`status`](Self::status) is `Translated`, and
    /// equal to the catalog's source locale when it is `Fallback`.
    pub locale: String,
    /// Whether `locale` reflects a real translation or a source-locale fallback.
    pub status: LocaleStatus,
    /// The literal text that renders for this fragment in this locale: the translated
    /// text when `status == Translated`, otherwise the catalog's canonical source text.
    /// Useful for a consumer doing text-content matching against rendered DOM nodes
    /// (see module docs, option 1).
    pub rendered_text: String,
}

/// Returns the project's declared source locale — the single locale every fragment in
/// an [`ApeironCatalog`] is authored in, prior to translation.
///
/// This is Glossa's whole-catalog/build-level locale: extraction (`universal_lift`)
/// does not attach a locale to individual items, so "the catalog's locale" is exactly
/// the project config's `source_locale` (see [`crate::config::LingoConfig`]), nothing
/// finer. For per-fragment locale information (which fragments have a translation into
/// a given target locale, and which fall back to this source locale), see [`locale_map`].
pub fn primary_locale(config: &crate::config::LingoConfig) -> &str {
    &config.source_locale
}

/// Builds a per-fragment locale map for one target-locale build.
///
/// For every item in `catalog`, looks up its translation record in `lock` (matched by
/// [`ApeironCoord`]) and determines, for `target_locale`:
///
/// - if a translation into `target_locale` exists in the lockfile: the fragment is
///   `Translated`, its effective locale is `target_locale`, and `rendered_text` is the
///   translated text;
/// - otherwise: the fragment is `Fallback`, its effective locale is `source_locale`
///   (the catalog's authored language — see [`primary_locale`]), and `rendered_text`
///   is the catalog's canonical source text.
///
/// `source_locale` is passed explicitly rather than re-derived, since a catalog alone
/// does not carry it (see module docs) — pass `primary_locale(config)`, or the
/// project's `LingoConfig::source_locale` directly.
///
/// The result is unordered by nothing in particular beyond `ApeironCatalog`'s own
/// `BTreeMap<ApeironCoord, _>` iteration order (deterministic, coordinate-sorted).
pub fn locale_map(
    catalog: &ApeironCatalog,
    lock: &LingoLock,
    target_locale: &str,
    source_locale: &str,
) -> Vec<LocaleRegion> {
    let mut out = Vec::with_capacity(catalog.items.len());

    for (coord, item) in &catalog.items {
        let locale_entry = lock
            .entries
            .get(coord)
            .and_then(|entry| entry.locales.get(target_locale));

        let (locale, status, rendered_text) = match locale_entry {
            Some(loc_entry) => (
                target_locale.to_string(),
                LocaleStatus::Translated,
                loc_entry.translated_text.clone(),
            ),
            None => (
                source_locale.to_string(),
                LocaleStatus::Fallback,
                item.canonical_text.clone(),
            ),
        };

        out.push(LocaleRegion {
            coord: *coord,
            locations: item.locations.clone(),
            key: item.key.clone(),
            locale,
            status,
            rendered_text,
        });
    }

    out
}

/// Computes the dominant (most frequent by fragment count) effective locale across a
/// set of [`LocaleRegion`]s, with ties broken by lexicographically smaller locale code
/// for determinism.
///
/// Directly useful for WCAG **SC 3.1.1**: a page's declared `<html lang>` should equal
/// this value, not merely equal whatever target locale the build was nominally
/// requested for — if most fragments fell back to the source locale (e.g. translation
/// coverage for `target_locale` is still low), the *actual* dominant language of the
/// page is the source locale, and declaring `lang="<target_locale>"` would itself be
/// an SC 3.1.1 violation despite being what was asked for at build time.
///
/// Returns `None` if `regions` is empty.
pub fn dominant_locale(regions: &[LocaleRegion]) -> Option<&str> {
    let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for region in regions {
        *counts.entry(region.locale.as_str()).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(a.0)))
        .map(|(locale, _)| locale)
}

/// Filters `regions` down to fragments whose effective locale differs from
/// `declared_page_lang`.
///
/// This is directly the WCAG **SC 3.1.2 Language of Parts** candidate set: each
/// returned region is a passage that, if `declared_page_lang` is what a rendered
/// page's `<html lang="...">` (or an ancestor element's `lang="..."`) declares, needs
/// its own `lang="<region.locale>"` attribute on whatever DOM element a consumer maps
/// it to (see module docs for how to find that element from `region.locations` /
/// `region.rendered_text`).
pub fn parts_needing_lang_attr<'a>(
    regions: &'a [LocaleRegion],
    declared_page_lang: &str,
) -> Vec<&'a LocaleRegion> {
    regions
        .iter()
        .filter(|r| r.locale != declared_page_lang)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::term::{ApeironTerm, ExtractedItem};
    use crate::lockfile::{LocaleEntry, LockEntry};

    fn make_item(text: &str, file: &str) -> ExtractedItem {
        let terms = vec![ApeironTerm::Text(text.to_string())];
        let coord = ApeironTerm::compute_coord(&terms);
        ExtractedItem {
            coord,
            key: None,
            terms,
            canonical_text: text.to_string(),
            context: None,
            locations: vec![SourceLocation {
                file_path: file.to_string(),
                line: 1,
                column: 1,
            }],
        }
    }

    #[test]
    fn translated_fragment_reports_target_locale() {
        let mut catalog = ApeironCatalog::new();
        let item = make_item("Hello", "src/App.tsx");
        let coord = item.coord;
        catalog.insert(item.clone());

        let mut lock = LingoLock::new();
        lock.entries.insert(
            coord,
            LockEntry {
                coord,
                source_hash: coord.raw(),
                canonical_text: item.canonical_text.clone(),
                context: None,
                locales: {
                    let mut m = std::collections::BTreeMap::new();
                    m.insert(
                        "fr".to_string(),
                        LocaleEntry {
                            translated_text: "Bonjour".to_string(),
                            translated_hash: 0,
                            provider: "offline".to_string(),
                            updated_at: "now".to_string(),
                            quality_score: 1.0,
                        },
                    );
                    m
                },
            },
        );

        let regions = locale_map(&catalog, &lock, "fr", "en");
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].locale, "fr");
        assert_eq!(regions[0].status, LocaleStatus::Translated);
        assert_eq!(regions[0].rendered_text, "Bonjour");
    }

    #[test]
    fn untranslated_fragment_falls_back_to_source_locale() {
        let mut catalog = ApeironCatalog::new();
        let item = make_item("Hello", "src/App.tsx");
        catalog.insert(item.clone());

        let lock = LingoLock::new(); // no translations recorded at all

        let regions = locale_map(&catalog, &lock, "fr", "en");
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].locale, "en");
        assert_eq!(regions[0].status, LocaleStatus::Fallback);
        assert_eq!(regions[0].rendered_text, "Hello");
    }

    #[test]
    fn parts_needing_lang_attr_finds_only_mismatches() {
        let mut catalog = ApeironCatalog::new();
        catalog.insert(make_item("Hello", "src/App.tsx"));
        catalog.insert(make_item("Untranslated stray string", "src/Widget.tsx"));

        let translated_coord =
            ApeironTerm::compute_coord(&[ApeironTerm::Text("Hello".to_string())]);
        let mut lock = LingoLock::new();
        lock.entries.insert(
            translated_coord,
            LockEntry {
                coord: translated_coord,
                source_hash: translated_coord.raw(),
                canonical_text: "Hello".to_string(),
                context: None,
                locales: {
                    let mut m = std::collections::BTreeMap::new();
                    m.insert(
                        "fr".to_string(),
                        LocaleEntry {
                            translated_text: "Bonjour".to_string(),
                            translated_hash: 0,
                            provider: "offline".to_string(),
                            updated_at: "now".to_string(),
                            quality_score: 1.0,
                        },
                    );
                    m
                },
            },
        );

        let regions = locale_map(&catalog, &lock, "fr", "en");
        let mismatches = parts_needing_lang_attr(&regions, "fr");
        assert_eq!(mismatches.len(), 1);
        assert_eq!(mismatches[0].rendered_text, "Untranslated stray string");
        assert_eq!(mismatches[0].locale, "en");
    }

    #[test]
    fn dominant_locale_reflects_low_translation_coverage() {
        let mut catalog = ApeironCatalog::new();
        catalog.insert(make_item("One", "src/A.tsx"));
        catalog.insert(make_item("Two", "src/B.tsx"));
        catalog.insert(make_item("Three untranslated stray text", "src/C.tsx"));

        // Only translate one of three fragments into "fr" -> source locale dominates.
        let one_coord = ApeironTerm::compute_coord(&[ApeironTerm::Text("One".to_string())]);
        let mut lock = LingoLock::new();
        lock.entries.insert(
            one_coord,
            LockEntry {
                coord: one_coord,
                source_hash: one_coord.raw(),
                canonical_text: "One".to_string(),
                context: None,
                locales: {
                    let mut m = std::collections::BTreeMap::new();
                    m.insert(
                        "fr".to_string(),
                        LocaleEntry {
                            translated_text: "Un".to_string(),
                            translated_hash: 0,
                            provider: "offline".to_string(),
                            updated_at: "now".to_string(),
                            quality_score: 1.0,
                        },
                    );
                    m
                },
            },
        );

        let regions = locale_map(&catalog, &lock, "fr", "en");
        assert_eq!(dominant_locale(&regions), Some("en"));
    }

    #[test]
    fn primary_locale_reads_config_source_locale() {
        let mut config = crate::config::LingoConfig::default();
        config.source_locale = "ja".to_string();
        assert_eq!(primary_locale(&config), "ja");
    }
}
