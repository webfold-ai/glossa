//! Zero-Runtime Static Pre-Compiler.
//!
//! Replaces translatable units in source files directly with the translated terms
//! for a target locale at compile time. Eliminates client-side i18n bundle size
//! and runtime lookup overhead entirely.

use crate::ir::term::ApeironTerm;
use std::collections::BTreeMap;

/// Replace extracted canonical text segments in the source code with their translated equivalents.
pub fn static_inline_source(
    source: &str,
    translations: &BTreeMap<String, Vec<ApeironTerm>>,
    target_framework: crate::emitters::TargetFramework,
) -> String {
    let mut result = source.to_string();

    // Sort keys by descending length to prevent partial prefix collisions
    let mut sorted_keys: Vec<&String> = translations.keys().collect();
    sorted_keys.sort_by_key(|k| std::cmp::Reverse(k.len()));

    for key in sorted_keys {
        if let Some(terms) = translations.get(key) {
            let inlined_replacement = match target_framework {
                crate::emitters::TargetFramework::React => {
                    crate::emitters::react::emit_react_inlined(terms)
                }
                crate::emitters::TargetFramework::Vue => {
                    crate::emitters::vue::emit_vue_inlined(terms)
                }
                crate::emitters::TargetFramework::Svelte => {
                    crate::emitters::svelte::emit_svelte_inlined(terms)
                }
                crate::emitters::TargetFramework::Solid => {
                    crate::emitters::solid::emit_solid_inlined(terms)
                }
                crate::emitters::TargetFramework::StaticZeroRuntime => {
                    ApeironTerm::to_canonical_string(terms)
                }
            };

            // Replace exact text occurrences
            result = result.replace(key, &inlined_replacement);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_inline() {
        let source = "<p>Welcome to our platform</p>";
        let mut trans = BTreeMap::new();
        trans.insert(
            "Welcome to our platform".to_string(),
            vec![ApeironTerm::Text(
                "Bienvenido a nuestra plataforma".to_string(),
            )],
        );

        let out = static_inline_source(source, &trans, crate::emitters::TargetFramework::React);
        assert_eq!(out, "<p>Bienvenido a nuestra plataforma</p>");
    }
}
