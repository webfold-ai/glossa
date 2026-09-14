//! Mitosis-Spirit Svelte Code Generator (fromTerm).
//!
//! Transforms translatable template nodes into standard Svelte i18n calls:
//! `$t("key")` or Paraglide `m.key()`.

use crate::ir::term::ApeironTerm;

/// Emits a Svelte template translation call for an Apeiron term.
pub fn emit_svelte_trans(key: &str, terms: &[ApeironTerm]) -> String {
    let slots = ApeironTerm::collect_slots(terms);

    if !slots.is_empty() {
        let mut out = format!("{{$t('{key}', {{ ");
        for (idx, slot) in slots.iter().enumerate() {
            if idx > 0 {
                out.push_str(", ");
            }
            out.push_str(slot);
        }
        out.push_str(" })}");
        out
    } else {
        format!("{{$t('{key}')}}")
    }
}

/// Inlines a translated text into Svelte template syntax.
pub fn emit_svelte_inlined(terms: &[ApeironTerm]) -> String {
    let mut out = String::new();
    for term in terms {
        match term {
            ApeironTerm::Text(s) => out.push_str(s),
            ApeironTerm::Slot { raw_expr, name, .. } => {
                out.push('{');
                out.push_str(raw_expr.as_deref().unwrap_or(name));
                out.push('}');
            }
            ApeironTerm::Tag { tag, attrs, children, .. } => {
                out.push('<');
                out.push_str(tag);
                for (k, v) in attrs {
                    out.push_str(&format!(" {k}=\"{v}\""));
                }
                out.push('>');
                out.push_str(&emit_svelte_inlined(children));
                out.push_str(&format!("</{tag}>"));
            }
            ApeironTerm::Plural { slot_name, cases } => {
                out.push_str(&format!("{{{slot_name} ? "));
                if let Some((_, branch)) = cases.first() {
                    out.push_str(&format!("`{}`", emit_svelte_inlined(branch)));
                }
                out.push_str(" : ''}");
            }
            ApeironTerm::Select { slot_name, cases } => {
                out.push_str(&format!("{{{slot_name} ? "));
                if let Some((_, branch)) = cases.first() {
                    out.push_str(&format!("`{}`", emit_svelte_inlined(branch)));
                }
                out.push_str(" : ''}");
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_svelte() {
        let terms = vec![
            ApeironTerm::Text("Items: ".to_string()),
            ApeironTerm::Slot {
                id: 0,
                name: "count".to_string(),
                raw_expr: Some("items.length".to_string()),
            },
        ];

        let call = emit_svelte_trans("cart.items", &terms);
        assert_eq!(call, "{$t('cart.items', { count })}");

        let inlined = emit_svelte_inlined(&terms);
        assert_eq!(inlined, "Items: {items.length}");
    }
}
