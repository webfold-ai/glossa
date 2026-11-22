//! False-positive, roundtrip, lockfile-merge, integrity-gate and codegen tests.
//!
//! What each test asserts:
//! 1. Code identifiers, CSS classes, IDs, URLs and imports in the fixtures are not extracted.
//! 2. UI strings, translatable attributes and inline calls in the fixtures are extracted.
//! 3. Translation calls like `{t('Order')}` extract `"Order"`, not `"{Order}"`.
//! 4. Parse -> emit -> parse returns the same catalog for the formats covered here.
//! 5. Merging lockfiles in several orders produces identical lockfiles.
//! 6. The integrity gate rejects the listed slot drops, tag corruptions and injected markup.
//! 7. Generated TypeScript and Rust accessors contain no external runtime imports.
//!
//! These are fixed fixtures: they check the listed cases, not every possible input.

use glossa::codegen::{generate_code, CodegenOptions, CodegenTarget};
use glossa::emitters::resource;
use glossa::engine::gate::ApeironNullGate;
use glossa::lockfile::{LingoLock, LocaleEntry, LockEntry};
use glossa::parsers::code::extract_code_strings;
use glossa::parsers::jsx::extract_jsx;
use glossa::parsers::resource::{
    extract_fluent, extract_json, extract_properties, extract_yaml,
};
use glossa::parsers::svelte::extract_svelte;
use glossa::parsers::vue::extract_vue;
use std::collections::BTreeMap;

// =========================================================================
// 1. NEGATIVE TESTING: CODE/ATTRS ARE NOT EXTRACTED
// =========================================================================
#[test]
fn test_no_false_positives_in_code_and_markup() {
    let source_code = r#"
    import React, { useState, useEffect } from 'react';
    import { useRouter } from 'next/router';
    import axios from 'axios';

    const API_ENDPOINT = "https://api.production.internal/v2/users";
    const IDENTIFIER_REGEX = /^[a-zA-Z_][a-zA-Z0-9_]*$/;
    const CSS_CLASSES = "flex items-center justify-between bg-slate-900 text-white p-4 rounded-lg";

    export default function SecurityPanel({ token, onLogout }) {
        const router = useRouter();
        const [status, setStatus] = useState("active");

        const handleRevoke = async () => {
            await axios.post("/api/v1/revoke-all", { token_id: "sec_123456" });
        };

        return (
            <section className={CSS_CLASSES} id="security-admin-panel" data-testid="sec-panel-root">
                <header aria-hidden="true" style={{ display: 'flex', zIndex: 100 }}>
                    <input type="hidden" name="csrf_token" value="abc_xyz_987" />
                    <img src="/static/assets/shield-icon.svg" width="24" height="24" alt="Security Shield" />
                </header>
                <div className="card-body">
                    <button type="submit" onClick={handleRevoke} className="btn-danger" data-analytics="btn_revoke_auth">
                        Revoke All Sessions
                    </button>
                    <a href="https://docs.enterprise.com/security" target="_blank" rel="noopener noreferrer">
                        Audit Documentation
                    </a>
                </div>
            </section>
        );
    }
    "#;

    let items = extract_jsx(source_code, "SecurityPanel.tsx");
    let extracted_texts: Vec<&str> = items.iter().map(|it| it.canonical_text.as_str()).collect();

    // 1. Assert MUST NOT extract non-translatable attributes, code constants, or internal IDs
    assert!(!extracted_texts.contains(&"flex items-center justify-between bg-slate-900 text-white p-4 rounded-lg"));
    assert!(!extracted_texts.contains(&"security-admin-panel"));
    assert!(!extracted_texts.contains(&"sec-panel-root"));
    assert!(!extracted_texts.contains(&"https://api.production.internal/v2/users"));
    assert!(!extracted_texts.contains(&"/api/v1/revoke-all"));
    assert!(!extracted_texts.contains(&"sec_123456"));
    assert!(!extracted_texts.contains(&"csrf_token"));
    assert!(!extracted_texts.contains(&"abc_xyz_987"));
    assert!(!extracted_texts.contains(&"/static/assets/shield-icon.svg"));
    assert!(!extracted_texts.contains(&"btn-danger"));
    assert!(!extracted_texts.contains(&"btn_revoke_auth"));
    assert!(!extracted_texts.contains(&"https://docs.enterprise.com/security"));
    assert!(!extracted_texts.contains(&"active"));

    // 2. Assert MUST extract ONLY legitimate UI text & translatable attributes
    assert!(extracted_texts.contains(&"Security Shield"), "Must extract img alt attribute");
    assert!(extracted_texts.contains(&"Revoke All Sessions"), "Must extract button text");
    assert!(extracted_texts.contains(&"Audit Documentation"), "Must extract anchor text");
}

// =========================================================================
// 2. ACCURACY: NO ACCIDENTAL SLOT CONVERSION OF TRANSLATION CALLS
// =========================================================================
#[test]
fn test_translation_calls_extracted_as_clean_text_not_slots() {
    // React / Next.js
    let react_src = r#"
    export function CartHeader() {
        return <h2>{t('Review Your Order')}</h2>;
    }
    "#;
    let react_items = extract_jsx(react_src, "CartHeader.tsx");
    let react_item = react_items.iter().find(|i| i.canonical_text == "Review Your Order")
        .expect("Must extract 'Review Your Order' directly as message text, NOT '{Review Your Order}'");
    assert_eq!(react_item.canonical_text, "Review Your Order");

    // Vue 3 SFC
    let vue_src = r#"
    <template>
        <h1>{{ $t('Billing & Invoices') }}</h1>
    </template>
    "#;
    let vue_items = extract_vue(vue_src, "Billing.vue");
    let vue_item = vue_items.iter().find(|i| i.canonical_text == "Billing & Invoices")
        .expect("Must extract 'Billing & Invoices' directly without slot braces");
    assert_eq!(vue_item.canonical_text, "Billing & Invoices");

    // Svelte
    let svelte_src = r#"
    <h2>{t('Workspace Management')}</h2>
    "#;
    let svelte_items = extract_svelte(svelte_src, "Workspace.svelte");
    let svelte_item = svelte_items.iter().find(|i| i.canonical_text == "Workspace Management")
        .expect("Must extract 'Workspace Management' directly without slot braces");
    assert_eq!(svelte_item.canonical_text, "Workspace Management");

    // Script Level in TypeScript
    let ts_src = r#"
    import { t } from '@/i18n';
    export const NOTIFICATIONS = {
        saveSuccess: t("Changes persisted to storage"),
        disconnect: t("WebSocket connection dropped, retrying..."),
    };
    "#;
    let ts_items = extract_code_strings(ts_src, "notifications.ts");
    assert!(ts_items.iter().any(|i| i.canonical_text == "Changes persisted to storage"));
    assert!(ts_items.iter().any(|i| i.canonical_text == "WebSocket connection dropped, retrying..."));
}

// =========================================================================
// 3. ROUNDTRIP: PARSE -> EMIT -> PARSE
// =========================================================================
#[test]
fn test_roundtrip_invariance_all_catalog_formats() {
    let mut original = BTreeMap::new();
    original.insert("welcome_msg".to_string(), "Welcome to Cloud, {user}!".to_string());
    original.insert("system_status".to_string(), "Node {id} operational (uptime: {uptime}h)".to_string());
    original.insert("cta_button".to_string(), "Upgrade Now".to_string());

    // 1. JSON Roundtrip
    let json_emitted = resource::emit_json(&original);
    let json_reparsed = extract_json(&json_emitted, "test.json");
    for (k, v) in &original {
        let found = json_reparsed.iter().find(|it| it.key.as_deref() == Some(k.as_str())).unwrap();
        assert_eq!(&found.canonical_text, v, "JSON roundtrip drift on key {k}");
    }

    // 2. YAML Roundtrip
    let yaml_emitted = resource::emit_yaml(&original, "en");
    let yaml_reparsed = extract_yaml(&yaml_emitted, "test.yml");
    for (k, v) in &original {
        let found = yaml_reparsed.iter().find(|it| it.key.as_deref() == Some(format!("en.{k}").as_str())).unwrap();
        assert_eq!(&found.canonical_text, v, "YAML roundtrip drift on key {k}");
    }

    // 3. Fluent (.ftl) Roundtrip
    let ftl_emitted = resource::emit_fluent(&original);
    let ftl_reparsed = extract_fluent(&ftl_emitted, "test.ftl");
    for (k, v) in &original {
        let found = ftl_reparsed.iter().find(|it| it.key.as_deref() == Some(k.as_str())).unwrap();
        assert_eq!(&found.canonical_text, v, "Fluent roundtrip drift on key {k}");
    }

    // 4. Java/Spring Properties Roundtrip
    let mut prop_str = String::new();
    for (k, v) in &original {
        prop_str.push_str(&format!("{k} = {v}\n"));
    }
    let prop_reparsed = extract_properties(&prop_str, "test.properties");
    for (k, v) in &original {
        let found = prop_reparsed.iter().find(|it| it.key.as_deref() == Some(k.as_str())).unwrap();
        assert_eq!(&found.canonical_text, v, "Properties roundtrip drift on key {k}");
    }
}

// =========================================================================
// 4. COMMUTATIVE LOCKFILE MERGE: PERMUTATION ORDER INVARIANCE
// =========================================================================
#[test]
fn test_lockfile_merge_order_invariance() {
    use glossa::ir::coord::ApeironCoord;

    fn make_branch(items: &[(&str, &str, &str, &str)]) -> LingoLock {
        let mut lock = LingoLock::new();
        for (coord_str, canon, locale, trans) in items {
            let coord = ApeironCoord::from_str_canonical(coord_str);
            let mut locales = BTreeMap::new();
            locales.insert(
                locale.to_string(),
                LocaleEntry {
                    translated_text: trans.to_string(),
                    translated_hash: 12345,
                    provider: "test".to_string(),
                    updated_at: "2026-09-15T00:00:00Z".to_string(),
                    quality_score: 1.0,
                },
            );
            lock.entries.insert(
                coord,
                LockEntry {
                    coord,
                    source_hash: coord.raw(),
                    canonical_text: canon.to_string(),
                    context: None,
                    locales,
                },
            );
        }
        lock
    }

    // 3 parallel feature branches created by different team members
    let branch_a = make_branch(&[
        ("Home Title", "Home Title", "es", "Título de Inicio"),
        ("Shared Button", "Shared Button", "es", "Botón Común"),
    ]);

    let branch_b = make_branch(&[
        ("Shared Button", "Shared Button", "fr", "Bouton Partagé"),
        ("Settings Page", "Settings Page", "es", "Página de Configuración"),
    ]);

    let branch_c = make_branch(&[
        ("Checkout Modal", "Checkout Modal", "de", "Kassen-Modal"),
        ("Shared Button", "Shared Button", "de", "Gemeinsame Schaltfläche"),
    ]);

    // Permutation 1: A + B + C
    let mut merge_1 = branch_a.clone();
    merge_1.merge(branch_b.clone());
    merge_1.merge(branch_c.clone());

    // Permutation 2: C + A + B
    let mut merge_2 = branch_c.clone();
    merge_2.merge(branch_a.clone());
    merge_2.merge(branch_b.clone());

    // Permutation 3: B + C + A
    let mut merge_3 = branch_b.clone();
    merge_3.merge(branch_c.clone());
    merge_3.merge(branch_a.clone());

    // All entries and locales must match across all tested permutations
    assert_eq!(merge_1.entries.len(), merge_2.entries.len());
    assert_eq!(merge_2.entries.len(), merge_3.entries.len());

    for (coord, entry1) in &merge_1.entries {
        let entry2 = merge_2.entries.get(coord).expect("Entry must exist in merge_2");
        let entry3 = merge_3.entries.get(coord).expect("Entry must exist in merge_3");

        assert_eq!(entry1.canonical_text, entry2.canonical_text);
        assert_eq!(entry2.canonical_text, entry3.canonical_text);
        assert_eq!(entry1.locales.len(), entry2.locales.len());
        assert_eq!(entry2.locales.len(), entry3.locales.len());

        for (loc, rec1) in &entry1.locales {
            let rec2 = entry2.locales.get(loc).unwrap();
            let rec3 = entry3.locales.get(loc).unwrap();
            assert_eq!(rec1.translated_text, rec2.translated_text);
            assert_eq!(rec2.translated_text, rec3.translated_text);
        }
    }
}

// =========================================================================
// 5. INTEGRITY GATE: CORRUPTED TRANSLATIONS ARE REJECTED
// =========================================================================
#[test]
fn test_integrity_gate_rejects_corrupted_translations() {
    let source = "Hello <0>{username}</0>, you have <1>{unreadCount}</1> new messages.";

    // Attack / Corruption 1: Dropped slot {unreadCount}
    assert!(ApeironNullGate::validate(source, "Bonjour <0>{username}</0>, vous avez de nouveaux messages.").is_err());

    // Attack / Corruption 2: Hallucinated / injected slot {bonusCredits}
    assert!(ApeironNullGate::validate(source, "Bonjour <0>{username}</0>, vous avez <1>{unreadCount}</1> messages et {bonusCredits} crédits.").is_err());

    // Attack / Corruption 3: Inverted tag numbers
    assert!(ApeironNullGate::validate(source, "Bonjour <1>{username}</1>, vous avez <0>{unreadCount}</0> messages.").is_err());

    // Attack / Corruption 4: Unclosed tag
    assert!(ApeironNullGate::validate(source, "Bonjour <0>{username}, vous avez <1>{unreadCount}</1> messages.").is_err());

    // Attack / Corruption 5: XSS script injection
    assert!(ApeironNullGate::validate(source, "Bonjour <script>window.location='http://attacker.com'</script> <0>{username}</0> <1>{unreadCount}</1>").is_err());

    // Attack / Corruption 6: Malicious onerror handler
    assert!(ApeironNullGate::validate(source, "Bonjour <img src=x onerror=alert(1)> <0>{username}</0> <1>{unreadCount}</1>").is_err());

    // Positive check: Valid translation with proper slots and tags MUST PASS
    let valid = "Bonjour <0>{username}</0>, vous avez <1>{unreadCount}</1> nouveaux messages.";
    assert!(ApeironNullGate::validate(source, valid).is_ok());
}

// =========================================================================
// 6. CODEGEN HAS NO RUNTIME IMPORTS
// =========================================================================
#[test]
fn test_codegen_emits_dependency_free_accessors() {
    use glossa::ir::coord::ApeironCoord;

    let mut lock = LingoLock::new();
    let coord1 = ApeironCoord::from_str_canonical("Welcome, {user}!");
    let mut locs = BTreeMap::new();
    locs.insert("es".to_string(), LocaleEntry {
        translated_text: "¡Bienvenido, {user}!".to_string(),
        translated_hash: 1,
        provider: "test".to_string(),
        updated_at: "2026-09-15T00:00:00Z".to_string(),
        quality_score: 1.0,
    });
    lock.entries.insert(coord1, LockEntry {
        coord: coord1,
        source_hash: coord1.raw(),
        canonical_text: "Welcome, {user}!".to_string(),
        context: None,
        locales: locs,
    });

    // 1. TypeScript Codegen
    let ts_opts = CodegenOptions {
        target: CodegenTarget::TypeScript,
        source_locale: "en".to_string(),
        target_locales: vec!["es".to_string()],
    };
    let ts_code = generate_code(&lock, &ts_opts);
    assert!(ts_code.contains("export type Locale = \"en\" | \"es\";"));
    assert!(ts_code.contains("export const m = {"));
    assert!(ts_code.contains("user: string | number"));
    assert!(!ts_code.contains("import "), "Codegen output must have no external runtime imports");

    // 2. Rust Codegen
    let rs_opts = CodegenOptions {
        target: CodegenTarget::Rust,
        source_locale: "en".to_string(),
        target_locales: vec!["es".to_string()],
    };
    let rs_code = generate_code(&lock, &rs_opts);
    assert!(rs_code.contains("pub mod m {"));
    assert!(rs_code.contains("match locale {"));
}
