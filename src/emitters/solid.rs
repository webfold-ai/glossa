//! Mitosis-Spirit Solid Code Generator (fromTerm).
//!
//! Transforms translatable template nodes into standard SolidJS i18n calls:
//! `t("key")`.

use crate::ir::term::ApeironTerm;

/// Emits a Solid translation call for an Apeiron term.
pub fn emit_solid_trans(key: &str, terms: &[ApeironTerm]) -> String {
    let slots = ApeironTerm::collect_slots(terms);

    if !slots.is_empty() {
        let mut out = format!("{{t('{key}', {{ ");
        for (idx, slot) in slots.iter().enumerate() {
            if idx > 0 {
                out.push_str(", ");
            }
            out.push_str(slot);
        }
        out.push_str(" })}}");
        out
    } else {
        format!("{{t('{key}')}}")
    }
}

/// Inlines a translated text into Solid JSX template syntax.
pub fn emit_solid_inlined(terms: &[ApeironTerm]) -> String {
    crate::emitters::react::emit_react_inlined(terms)
}
