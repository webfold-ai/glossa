//! Cross-framework emitters (`fromTerm`).
//!
//! Compiles ApeironTerm units into React, Vue, Svelte and Solid output or static evaluations.

pub mod react;
pub mod resource;
pub mod solid;
pub mod static_eval;
pub mod svelte;
pub mod vue;

use crate::ir::term::ApeironTerm;

/// Target framework for code generation.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TargetFramework {
    React,
    Vue,
    Svelte,
    Solid,
    StaticZeroRuntime,
}

impl TargetFramework {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "react" | "next" | "nextjs" => Some(Self::React),
            "vue" | "nuxt" => Some(Self::Vue),
            "svelte" | "sveltekit" => Some(Self::Svelte),
            "solid" | "solidstart" => Some(Self::Solid),
            "static" | "zero" | "zero-runtime" => Some(Self::StaticZeroRuntime),
            _ => None,
        }
    }
}

/// Emits framework-native translation invocation for a term.
pub fn emit_trans_call(key: &str, terms: &[ApeironTerm], target: TargetFramework) -> String {
    match target {
        TargetFramework::React => react::emit_react_trans(key, terms),
        TargetFramework::Vue => vue::emit_vue_trans(key, terms),
        TargetFramework::Svelte => svelte::emit_svelte_trans(key, terms),
        TargetFramework::Solid => solid::emit_solid_trans(key, terms),
        TargetFramework::StaticZeroRuntime => ApeironTerm::to_canonical_string(terms),
    }
}
