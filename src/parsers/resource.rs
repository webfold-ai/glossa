//! High-Speed Resource Catalog Parser (JSON, PO, Fluent).
//!
//! Lifts traditional localization files directly into the Apeiron basis,
//! preserving existing translation keys and string namespaces.

use crate::ir::coord::ApeironCoord;
use crate::ir::term::{ApeironTerm, ExtractedItem, SourceLocation};
use serde_json::Value;

/// Extracts translatable keys and values from JSON (flat or nested).
pub fn extract_json(source: &str, file_path: &str) -> Vec<ExtractedItem> {
    let mut items = Vec::new();
    if let Ok(value) = serde_json::from_str::<Value>(source) {
        flatten_json(&value, String::new(), file_path, &mut items);
    }
    items
}

fn flatten_json(val: &Value, prefix: String, file_path: &str, items: &mut Vec<ExtractedItem>) {
    match val {
        Value::Object(map) => {
            for (k, v) in map {
                let next_key = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{prefix}.{k}")
                };
                flatten_json(v, next_key, file_path, items);
            }
        }
        Value::String(s) => {
            if !s.trim().is_empty() {
                let parsed_terms = parse_interpolated_string(s);
                let canon = ApeironTerm::to_canonical_string(&parsed_terms);
                let coord = ApeironCoord::from_str_canonical(&canon);
                items.push(ExtractedItem {
                    coord,
                    key: Some(prefix),
                    terms: parsed_terms,
                    canonical_text: canon,
                    context: Some(format!("JSON in {file_path}")),
                    locations: vec![SourceLocation {
                        file_path: file_path.to_string(),
                        line: 1,
                        column: 1,
                    }],
                });
            }
        }
        _ => {}
    }
}

/// Extracts translatable entries from GNU gettext PO files.
pub fn extract_po(source: &str, file_path: &str) -> Vec<ExtractedItem> {
    let mut items = Vec::new();
    let mut current_msgid = String::new();
    let mut current_msgctxt: Option<String> = None;
    let mut reading_msgid = false;
    let mut line_num = 0;

    for (idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }

        if trimmed.starts_with("msgctxt ") {
            if !current_msgid.is_empty() {
                flush_po_item(&mut items, &current_msgid, current_msgctxt.as_deref(), file_path, line_num);
                current_msgid.clear();
            }
            current_msgctxt = extract_quoted_string(trimmed);
            reading_msgid = false;
        } else if trimmed.starts_with("msgid ") {
            if !current_msgid.is_empty() {
                flush_po_item(&mut items, &current_msgid, current_msgctxt.as_deref(), file_path, line_num);
                current_msgid.clear();
            }
            line_num = idx + 1;
            if let Some(s) = extract_quoted_string(trimmed) {
                current_msgid.push_str(&s);
            }
            reading_msgid = true;
        } else if trimmed.starts_with("msgstr ") {
            reading_msgid = false;
        } else if reading_msgid && trimmed.starts_with('"') {
            if let Some(s) = extract_quoted_string(trimmed) {
                current_msgid.push_str(&s);
            }
        }
    }

    if !current_msgid.is_empty() {
        flush_po_item(&mut items, &current_msgid, current_msgctxt.as_deref(), file_path, line_num);
    }

    items
}

fn flush_po_item(
    items: &mut Vec<ExtractedItem>,
    msgid: &str,
    context: Option<&str>,
    file_path: &str,
    line_num: usize,
) {
    if msgid.trim().is_empty() {
        return; // skip PO header
    }
    let parsed_terms = parse_interpolated_string(msgid);
    let canon = ApeironTerm::to_canonical_string(&parsed_terms);
    let coord = ApeironCoord::from_str_canonical(&canon);
    items.push(ExtractedItem {
        coord,
        key: Some(msgid.to_string()),
        terms: parsed_terms,
        canonical_text: canon,
        context: context.map(|c| c.to_string()).or_else(|| Some(format!("PO in {file_path}:{line_num}"))),
        locations: vec![SourceLocation {
            file_path: file_path.to_string(),
            line: line_num,
            column: 1,
        }],
    });
}

/// Extracts translatable messages from Fluent (.ftl) files.
pub fn extract_fluent(source: &str, file_path: &str) -> Vec<ExtractedItem> {
    let mut items = Vec::new();
    for (idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        if let Some((key, val)) = trimmed.split_once('=') {
            let key = key.trim().to_string();
            let val = val.trim();
            if !val.is_empty() {
                let parsed_terms = parse_fluent_value(val);
                let canon = ApeironTerm::to_canonical_string(&parsed_terms);
                let coord = ApeironCoord::from_str_canonical(&canon);
                items.push(ExtractedItem {
                    coord,
                    key: Some(key),
                    terms: parsed_terms,
                    canonical_text: canon,
                    context: Some(format!("Fluent in {file_path}:{}", idx + 1)),
                    locations: vec![SourceLocation {
                        file_path: file_path.to_string(),
                        line: idx + 1,
                        column: 1,
                    }],
                });
            }
        }
    }
    items
}

fn extract_quoted_string(line: &str) -> Option<String> {
    let first = line.find('"')?;
    let last = line.rfind('"')?;
    if first < last {
        Some(line[first + 1..last].to_string())
    } else {
        None
    }
}

/// Parses string with `{var}`, `{{var}}`, or `%s` placeholders into ApeironTerm.
pub fn parse_interpolated_string(s: &str) -> Vec<ApeironTerm> {
    let mut terms = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut buf = String::new();
    let mut slot_id = 0;

    while i < chars.len() {
        // Double curly `{{name}}`
        if chars[i] == '{' && i + 1 < chars.len() && chars[i + 1] == '{' {
            if !buf.is_empty() {
                terms.push(ApeironTerm::Text(buf.clone()));
                buf.clear();
            }
            i += 2;
            let mut name = String::new();
            while i + 1 < chars.len() && !(chars[i] == '}' && chars[i + 1] == '}') {
                name.push(chars[i]);
                i += 1;
            }
            if i + 1 < chars.len() {
                i += 2;
            }
            let trimmed = name.trim();
            terms.push(ApeironTerm::Slot {
                id: slot_id,
                name: if trimmed.is_empty() { format!("var{slot_id}") } else { trimmed.to_string() },
                raw_expr: Some(trimmed.to_string()),
            });
            slot_id += 1;
            continue;
        }

        // Single curly `{name}`
        if chars[i] == '{' {
            if !buf.is_empty() {
                terms.push(ApeironTerm::Text(buf.clone()));
                buf.clear();
            }
            i += 1;
            let mut name = String::new();
            while i < chars.len() && chars[i] != '}' {
                name.push(chars[i]);
                i += 1;
            }
            if i < chars.len() {
                i += 1;
            }
            let trimmed = name.trim();
            terms.push(ApeironTerm::Slot {
                id: slot_id,
                name: if trimmed.is_empty() { format!("var{slot_id}") } else { trimmed.to_string() },
                raw_expr: Some(trimmed.to_string()),
            });
            slot_id += 1;
            continue;
        }

        // C-style printf `%s` or `%d`
        if chars[i] == '%' && i + 1 < chars.len() && (chars[i + 1] == 's' || chars[i + 1] == 'd') {
            if !buf.is_empty() {
                terms.push(ApeironTerm::Text(buf.clone()));
                buf.clear();
            }
            i += 2;
            terms.push(ApeironTerm::Slot {
                id: slot_id,
                name: format!("var{slot_id}"),
                raw_expr: None,
            });
            slot_id += 1;
            continue;
        }

        buf.push(chars[i]);
        i += 1;
    }

    if !buf.is_empty() {
        terms.push(ApeironTerm::Text(buf));
    }

    terms
}

fn parse_fluent_value(s: &str) -> Vec<ApeironTerm> {
    // In Fluent, variables are {$var}
    let replaced = s.replace("{$", "{");
    parse_interpolated_string(&replaced)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json() {
        let json = r#"{
            "common": {
                "greeting": "Hello, {name}!",
                "items_count": "You have {{count}} items in your cart."
            }
        }"#;

        let items = extract_json(json, "en.json");
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|it| it.canonical_text == "Hello, {name}!"));
        assert!(items.iter().any(|it| it.canonical_text == "You have {count} items in your cart."));
    }

    #[test]
    fn test_extract_po() {
        let po = r#"
        msgid ""
        msgstr "Content-Type: text/plain; charset=UTF-8\n"

        msgctxt "nav"
        msgid "Settings"
        msgstr ""

        msgid "User %s has %d points"
        msgstr ""
        "#;

        let items = extract_po(po, "messages.po");
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|it| it.canonical_text == "Settings"));
        assert!(items.iter().any(|it| it.canonical_text == "User {var0} has {var1} points"));
    }
}
