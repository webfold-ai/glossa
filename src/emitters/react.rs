//! Mitosis-Spirit React/Next.js Code Generator (fromTerm).
//!
//! Transforms translatable JSX nodes into standard React i18n calls:
//! `<Trans i18nKey="key" values={{...}} />` or `t("key")`.

use crate::ir::term::ApeironTerm;

/// Emits a React JSX translation expression or component for an Apeiron term.
pub fn emit_react_trans(key: &str, terms: &[ApeironTerm]) -> String {
    let slots = ApeironTerm::collect_slots(terms);
    let has_tags = terms.iter().any(|t| matches!(t, ApeironTerm::Tag { .. }));

    if has_tags {
        // Use <Trans /> component for complex markup
        let mut out = format!("<Trans i18nKey=\"{key}\"");
        if !slots.is_empty() {
            out.push_str(" values={{ ");
            for (idx, slot) in slots.iter().enumerate() {
                if idx > 0 {
                    out.push_str(", ");
                }
                out.push_str(slot);
            }
            out.push_str(" }}");
        }
        out.push_str(" />");
        out
    } else if !slots.is_empty() {
        // Use t('key', { var1, var2 })
        let mut out = format!("{{t('{key}', {{ ");
        for (idx, slot) in slots.iter().enumerate() {
            if idx > 0 {
                out.push_str(", ");
            }
            out.push_str(slot);
        }
        out.push_str(" })}");
        out
    } else {
        // Plain text: {t('key')}
        format!("{{t('{key}')}}")
    }
}

/// Inlines a translated text into React JSX, handling tags and slot substitutions.
pub fn emit_react_inlined(terms: &[ApeironTerm]) -> String {
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
                out.push_str(&emit_react_inlined(children));
                out.push_str(&format!("</{tag}>"));
            }
            ApeironTerm::Plural { slot_name, cases } => {
                out.push_str(&format!("{{{slot_name} ? "));
                if let Some((_, branch)) = cases.first() {
                    out.push_str(&format!("`{}`", emit_react_inlined(branch)));
                }
                out.push_str(" : ''}");
            }
            ApeironTerm::Select { slot_name, cases } => {
                out.push_str(&format!("{{{slot_name} ? "));
                if let Some((_, branch)) = cases.first() {
                    out.push_str(&format!("`{}`", emit_react_inlined(branch)));
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
    fn test_emit_react() {
        let terms = vec![
            ApeironTerm::Text("Hello ".to_string()),
            ApeironTerm::Slot {
                id: 0,
                name: "user".to_string(),
                raw_expr: Some("user.name".to_string()),
            },
            ApeironTerm::Text("!".to_string()),
        ];

        let call = emit_react_trans("common.hello", &terms);
        assert_eq!(call, "{t('common.hello', { user })}");

        let inlined = emit_react_inlined(&terms);
        assert_eq!(inlined, "Hello {user.name}!");
    }
}
