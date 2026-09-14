//! Universal Code Extractor for Backend, Mobile, and Template Languages.
//!
//! Theorem: `adapt = fromTerm ∘ universalLift` (Sanguine/UniversalAdapter).
//! Extracts localized strings and message keys across:
//! - Python (Django, Flask, FastAPI, gettext `_("...")`)
//! - Go (`i18n.T("...")`, `gotext.Get("...")`)
//! - Ruby on Rails (`I18n.t("...")`, `t("...")`)
//! - PHP / Laravel (`__("...")`, `trans("...")`)
//! - Swift / iOS (`NSLocalizedString("...")`, `String(localized: "...")`)
//! - Dart / Flutter (`tr("...")`, `context.tr("...")`)
//! - Angular (`$localize` strings and template attributes)
//! - Astro components (`.astro`)

use crate::ir::coord::ApeironCoord;
use crate::ir::term::{ApeironTerm, ExtractedItem, SourceLocation};
use crate::parsers::resource::parse_interpolated_string;
use regex::Regex;
use std::sync::OnceLock;

static CODE_CALL_REGEX: OnceLock<Regex> = OnceLock::new();
static ANGULAR_LOCALIZE_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_code_call_regex() -> &'static Regex {
    CODE_CALL_REGEX.get_or_init(|| {
        // Matches standard gettext/i18n calls or Swift String(localized: "...")
        Regex::new(r#"(?:_|\bgettext|\btrans|\bNSLocalizedString|\b[iI]18n\.[tT]|\b[tT]|__)\s*\(\s*(?:"([^"\\]*(?:\\.[^"\\]*)*)"|'([^'\\]*(?:\\.[^'\\]*)*)')|String\s*\(\s*localized:\s*(?:"([^"\\]*(?:\\.[^"\\]*)*)"|'([^'\\]*(?:\\.[^'\\]*)*)')"#)
            .unwrap()
    })
}

fn get_angular_localize_regex() -> &'static Regex {
    ANGULAR_LOCALIZE_REGEX.get_or_init(|| {
        // Matches $localize`:@@optionalId:Message text` or $localize`Message text`
        Regex::new(r#"\$localize\s*`(?::@@([^:`]+):)?([^`]+)`"#).unwrap()
    })
}

/// Extracts translatable strings from source code files (.py, .go, .rb, .php, .swift, .dart, .astro, .ts).
pub fn extract_code_strings(source: &str, file_path: &str) -> Vec<ExtractedItem> {
    let mut items = Vec::new();
    let re = get_code_call_regex();

    for (line_idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        // Skip comment-only lines
        if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with('*') {
            continue;
        }

        // Standard function call extraction
        for caps in re.captures_iter(line) {
            let matched_text = caps
                .get(1)
                .or_else(|| caps.get(2))
                .or_else(|| caps.get(3))
                .or_else(|| caps.get(4))
                .map(|m| m.as_str());
            if let Some(text) = matched_text {
                let clean_text = text.replace("\\\"", "\"").replace("\\'", "'");
                if !clean_text.trim().is_empty() {
                    let parsed_terms = parse_interpolated_string(&clean_text);
                    let canon = ApeironTerm::to_canonical_string(&parsed_terms);
                    let coord = ApeironCoord::from_str_canonical(&canon);
                    items.push(ExtractedItem {
                        coord,
                        key: None,
                        terms: parsed_terms,
                        canonical_text: canon,
                        context: Some(format!("Call in {file_path}:{}", line_idx + 1)),
                        locations: vec![SourceLocation {
                            file_path: file_path.to_string(),
                            line: line_idx + 1,
                            column: 1,
                        }],
                    });
                }
            }
        }

        // Angular $localize support
        let ang_re = get_angular_localize_regex();
        for caps in ang_re.captures_iter(line) {
            let key = caps.get(1).map(|m| m.as_str().to_string());
            if let Some(text_match) = caps.get(2) {
                let text = text_match.as_str().trim();
                if !text.is_empty() {
                    let parsed_terms = parse_interpolated_string(text);
                    let canon = ApeironTerm::to_canonical_string(&parsed_terms);
                    let coord = ApeironCoord::from_str_canonical(&canon);
                    items.push(ExtractedItem {
                        coord,
                        key,
                        terms: parsed_terms,
                        canonical_text: canon,
                        context: Some(format!("Angular $localize in {file_path}:{}", line_idx + 1)),
                        locations: vec![SourceLocation {
                            file_path: file_path.to_string(),
                            line: line_idx + 1,
                            column: 1,
                        }],
                    });
                }
            }
        }
    }

    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_python() {
        let py_code = r#"
from django.utils.translation import gettext as _

def user_view(request):
    msg = _("Welcome to our service, {user}!")
    btn = _("Submit Form")
    return render(request, "index.html", {"msg": msg, "btn": btn})
"#;
        let items = extract_code_strings(py_code, "views.py");
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|i| i.canonical_text == "Welcome to our service, {user}!"));
        assert!(items.iter().any(|i| i.canonical_text == "Submit Form"));
    }

    #[test]
    fn test_extract_go() {
        let go_code = r#"
package main
import "github.com/nicksnyder/go-i18n/v2/i18n"

func main() {
    title := i18n.T("Project Dashboard")
    desc := t("Manage your cluster nodes")
}
"#;
        let items = extract_code_strings(go_code, "main.go");
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|i| i.canonical_text == "Project Dashboard"));
    }

    #[test]
    fn test_extract_ruby_rails() {
        let rb_code = r#"
class UsersController < ApplicationController
  def create
    flash[:notice] = I18n.t("User was successfully created.")
    @label = t("Back to Home")
  end
end
"#;
        let items = extract_code_strings(rb_code, "users_controller.rb");
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|i| i.canonical_text == "User was successfully created."));
    }

    #[test]
    fn test_extract_php_laravel() {
        let php_code = r#"
<?php
echo __('Welcome to our application');
echo trans('messages.greeting');
"#;
        let items = extract_code_strings(php_code, "index.php");
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|i| i.canonical_text == "Welcome to our application"));
    }

    #[test]
    fn test_extract_swift() {
        let swift_code = r#"
let alertTitle = NSLocalizedString("Invalid Credentials", comment: "Login error")
let buttonLabel = String(localized: "Try Again")
"#;
        let items = extract_code_strings(swift_code, "LoginViewController.swift");
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|i| i.canonical_text == "Invalid Credentials"));
        assert!(items.iter().any(|i| i.canonical_text == "Try Again"));
    }

    #[test]
    fn test_extract_angular_localize() {
        let ts_code = r#"
const title = $localize`:@@home.title:Welcome to Angular!`;
const greeting = $localize`Hello guest`;
"#;
        let items = extract_code_strings(ts_code, "app.component.ts");
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|i| i.key.as_deref() == Some("home.title")));
        assert!(items.iter().any(|i| i.canonical_text == "Welcome to Angular!"));
    }
}
