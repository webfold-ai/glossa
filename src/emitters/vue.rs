//! Mitosis-Spirit Vue Code Generator (fromTerm).
//!
//! Transforms translatable template nodes into standard Vue i18n calls:
//! `<i18n-t keypath="key">` or `$t("key")`.

use crate::ir::term::ApeironTerm;

/// Emits a Vue template translation call for an Apeiron term.
pub fn emit_vue_trans(key: &str, terms: &[ApeironTerm]) -> String {
    let slots = ApeironTerm::collect_slots(terms);
    let has_tags = terms.iter().any(|t| matches!(t, ApeironTerm::Tag { .. }));

    if has_tags {
        let mut out = format!("<i18n-t keypath=\"{key}\" tag=\"span\"");
        if !slots.is_empty() {
            out.push_str(" :values=\"{ ");
            for (idx, slot) in slots.iter().enumerate() {
                if idx > 0 {
                    out.push_str(", ");
                }
                out.push_str(&format!("{slot}: {slot}"));
            }
            out.push_str(" }\"");
        }
        out.push_str(" />");
        out
    } else if !slots.is_empty() {
        let mut out = format!("{{{{ $t('{key}', {{ ");
        for (idx, slot) in slots.iter().enumerate() {
            if idx > 0 {
                out.push_str(", ");
            }
            out.push_str(&format!("{slot}: {slot}"));
        }
        out.push_str(" }) }}");
        out
    } else {
        format!("{{{{ $t('{key}') }}}}")
    }
}

/// Inlines a translated text into Vue template syntax.
pub fn emit_vue_inlined(terms: &[ApeironTerm]) -> String {
    let mut out = String::new();
    for term in terms {
        match term {
            ApeironTerm::Text(s) => out.push_str(s),
            ApeironTerm::Slot { raw_expr, name, .. } => {
                out.push_str("{{ ");
                out.push_str(raw_expr.as_deref().unwrap_or(name));
                out.push_str(" }}");
            }
            ApeironTerm::Tag {
                tag,
                attrs,
                children,
                ..
            } => {
                out.push('<');
                out.push_str(tag);
                for (k, v) in attrs {
                    out.push_str(&format!(" {k}=\"{v}\""));
                }
                out.push('>');
                out.push_str(&emit_vue_inlined(children));
                out.push_str(&format!("</{tag}>"));
            }
            ApeironTerm::Plural { slot_name, cases } => {
                out.push_str(&format!("{{{{ {slot_name} ? "));
                if let Some((_, branch)) = cases.first() {
                    out.push_str(&format!("'{}'", emit_vue_inlined(branch)));
                }
                out.push_str(" : '' }}}}");
            }
            ApeironTerm::Select { slot_name, cases } => {
                out.push_str(&format!("{{{{ {slot_name} ? "));
                if let Some((_, branch)) = cases.first() {
                    out.push_str(&format!("'{}'", emit_vue_inlined(branch)));
                }
                out.push_str(" : '' }}}}");
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_vue() {
        let terms = vec![
            ApeironTerm::Text("Welcome ".to_string()),
            ApeironTerm::Slot {
                id: 0,
                name: "user".to_string(),
                raw_expr: Some("user.name".to_string()),
            },
        ];

        let call = emit_vue_trans("home.welcome", &terms);
        assert_eq!(call, "{{ $t('home.welcome', { user: user }) }}");

        let inlined = emit_vue_inlined(&terms);
        assert_eq!(inlined, "Welcome {{ user.name }}");
    }
}
