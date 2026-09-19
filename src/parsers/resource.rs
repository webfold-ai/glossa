//! High-Speed Universal Resource Catalog Parser.
//! Supports: JSON, YAML, GNU gettext PO, Mozilla Fluent, Android XML, iOS .strings,
//! Flutter ARB, Java/Kotlin .properties, XLIFF 1.2/2.0, and CSV/TSV.
//!
//! Lifts traditional localization files directly into the Apeiron basis,
//! preserving existing translation keys, variable slots, and namespaces.

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

/// Extracts translatable entries from HuggingFace Dataset JSON Lines (.jsonl).
pub fn extract_jsonl(source: &str, file_path: &str) -> Vec<ExtractedItem> {
    let mut items = Vec::new();
    for (line_idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(trimmed) {
            let text_val = map
                .get("source")
                .or_else(|| map.get("text"))
                .or_else(|| map.get("translation"))
                .and_then(|v| v.as_str());

            if let Some(text) = text_val {
                if !text.trim().is_empty() {
                    let parsed_terms = parse_interpolated_string(text);
                    let canon = ApeironTerm::to_canonical_string(&parsed_terms);
                    let coord = ApeironCoord::from_str_canonical(&canon);
                    items.push(ExtractedItem {
                        coord,
                        key: map.get("key").and_then(|k| k.as_str()).map(|k| k.to_string()),
                        terms: parsed_terms,
                        canonical_text: canon,
                        context: Some(format!("JSONL in {file_path}:{}", line_idx + 1)),
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

/// Extracts translatable entries from Flutter ARB (.arb) files.
/// ARB is JSON where keys starting with `@` are attribute metadata.
pub fn extract_flutter_arb(source: &str, file_path: &str) -> Vec<ExtractedItem> {
    let mut items = Vec::new();
    if let Ok(Value::Object(map)) = serde_json::from_str::<Value>(source) {
        for (k, v) in map {
            if k.starts_with('@') {
                continue; // Skip metadata objects like `@hello`: { "description": "..." }
            }
            if let Value::String(s) = v {
                if !s.trim().is_empty() {
                    let parsed_terms = parse_interpolated_string(&s);
                    let canon = ApeironTerm::to_canonical_string(&parsed_terms);
                    let coord = ApeironCoord::from_str_canonical(&canon);
                    items.push(ExtractedItem {
                        coord,
                        key: Some(k),
                        terms: parsed_terms,
                        canonical_text: canon,
                        context: Some(format!("Flutter ARB in {file_path}")),
                        locations: vec![SourceLocation {
                            file_path: file_path.to_string(),
                            line: 1,
                            column: 1,
                        }],
                    });
                }
            }
        }
    }
    items
}

/// Extracts translatable entries from YAML files (.yml, .yaml).
pub fn extract_yaml(source: &str, file_path: &str) -> Vec<ExtractedItem> {
    let mut items = Vec::new();
    let mut stack: Vec<(usize, String)> = Vec::new();

    for (idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("---") {
            continue;
        }

        let indent = line.len() - line.trim_start().len();

        // Pop keys that are at or deeper than current indentation
        while let Some((last_indent, _)) = stack.last() {
            if *last_indent >= indent {
                stack.pop();
            } else {
                break;
            }
        }

        if let Some((k, v)) = trimmed.split_once(':') {
            let key = k.trim().trim_matches('\'').trim_matches('"');
            let val = v.trim();

            if val.is_empty() {
                // Section header (nesting)
                stack.push((indent, key.to_string()));
            } else {
                // Leaf value
                let clean_val = val.trim_matches('\'').trim_matches('"');
                if !clean_val.is_empty() {
                    let full_key = if stack.is_empty() {
                        key.to_string()
                    } else {
                        let prefix = stack.iter().map(|(_, s)| s.as_str()).collect::<Vec<_>>().join(".");
                        format!("{prefix}.{key}")
                    };

                    let parsed_terms = parse_interpolated_string(clean_val);
                    let canon = ApeironTerm::to_canonical_string(&parsed_terms);
                    let coord = ApeironCoord::from_str_canonical(&canon);
                    items.push(ExtractedItem {
                        coord,
                        key: Some(full_key),
                        terms: parsed_terms,
                        canonical_text: canon,
                        context: Some(format!("YAML in {file_path}:{}", idx + 1)),
                        locations: vec![SourceLocation {
                            file_path: file_path.to_string(),
                            line: idx + 1,
                            column: indent + 1,
                        }],
                    });
                }
            }
        }
    }

    items
}

/// Extracts translatable entries from Android XML (`res/values/strings.xml`).
/// Decodes standard XML entities and numeric character references in a
/// single left-to-right pass (not sequential `.replace()` calls, which would
/// risk double-decoding e.g. a literal `&amp;lt;` into `<`).
///
/// Real production Android `strings.xml` files lean on numeric references
/// heavily — verified against `mendhak/gpslogger`'s real strings.xml, which
/// has 18 of them (`&#160;` non-breaking space ×13, `&#176;` °, `&#189;` ½)
/// and zero named entities. Before this fix, `extract_android_xml` only
/// unescaped Android's own backslash escapes (`\'`, `\"`, `\n`) and left
/// every XML entity as literal text — so `&#160;foot` extracted as the
/// 9-character string `&#160;foot` instead of the intended non-breaking
/// space followed by "foot".
fn decode_xml_entities(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c != '&' {
            out.push(c);
            continue;
        }
        let rest = &input[i..];
        if let Some(end) = rest.find(';') {
            if end <= 10 {
                let entity = &rest[1..end];
                let decoded = match entity {
                    "amp" => Some('&'),
                    "lt" => Some('<'),
                    "gt" => Some('>'),
                    "quot" => Some('"'),
                    "apos" => Some('\''),
                    _ if entity.starts_with('#') => {
                        let code = if let Some(hex) = entity[1..].strip_prefix('x').or_else(|| entity[1..].strip_prefix('X')) {
                            u32::from_str_radix(hex, 16).ok()
                        } else {
                            entity[1..].parse::<u32>().ok()
                        };
                        code.and_then(char::from_u32)
                    }
                    _ => None,
                };
                if let Some(ch) = decoded {
                    out.push(ch);
                    // Skip the consumed characters (everything up to and including ';').
                    for _ in 0..end {
                        chars.next();
                    }
                    continue;
                }
            }
        }
        // Not a recognized entity — keep the literal '&'.
        out.push('&');
    }
    out
}

pub fn extract_android_xml(source: &str, file_path: &str) -> Vec<ExtractedItem> {
    let mut items = Vec::new();

    // Fast regex / slice matching for <string name="...">value</string>
    let mut cursor = 0;
    while let Some(start_tag) = source[cursor..].find("<string") {
        let actual_start = cursor + start_tag;
        if let Some(close_tag) = source[actual_start..].find('>') {
            let tag_header = &source[actual_start..actual_start + close_tag];
            let name_key = extract_attribute(tag_header, "name");

            let content_start = actual_start + close_tag + 1;
            if let Some(end_tag) = source[content_start..].find("</string>") {
                let content = &source[content_start..content_start + end_tag];
                // Trim incidental XML pretty-print whitespace from the RAW
                // content first, then decode entities — decoding before
                // trimming would let `str::trim()` (which strips any
                // Unicode-whitespace char, not just ASCII) eat a leading/
                // trailing non-breaking space (&#160; -> U+00A0) that was
                // meaningful decoded content, not source formatting.
                let clean_content = decode_xml_entities(
                    &content.trim().replace("\\'", "'").replace("\\\"", "\"").replace("\\n", "\n"),
                );

                if !clean_content.is_empty() {
                    let parsed_terms = parse_interpolated_string(&clean_content);
                    let canon = ApeironTerm::to_canonical_string(&parsed_terms);
                    let coord = ApeironCoord::from_str_canonical(&canon);
                    items.push(ExtractedItem {
                        coord,
                        key: name_key,
                        terms: parsed_terms,
                        canonical_text: canon,
                        context: Some(format!("Android XML in {file_path}")),
                        locations: vec![SourceLocation {
                            file_path: file_path.to_string(),
                            line: 1,
                            column: 1,
                        }],
                    });
                }
                cursor = content_start + end_tag + 9;
                continue;
            }
        }
        cursor = actual_start + 7;
    }

    items
}

/// Extracts translatable entries from Apple iOS / macOS `.strings` files.
pub fn extract_ios_strings(source: &str, file_path: &str) -> Vec<ExtractedItem> {
    let mut items = Vec::new();
    let mut in_comment = false;

    for (idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if in_comment {
            if trimmed.contains("*/") {
                in_comment = false;
            }
            continue;
        }
        if trimmed.starts_with("/*") {
            if !trimmed.contains("*/") {
                in_comment = true;
            }
            continue;
        }
        if trimmed.starts_with("//") {
            continue;
        }

        // Format: "key" = "value";
        if let Some((k_part, v_part)) = trimmed.split_once('=') {
            let key = k_part.trim().trim_matches('"');
            let val = v_part.trim().trim_end_matches(';').trim().trim_matches('"');

            if !val.is_empty() {
                // Convert iOS format specifiers (%@, %ld, %d) to Apeiron slots
                let normalized_val = val.replace("%@", "{var}");
                let parsed_terms = parse_interpolated_string(&normalized_val);
                let canon = ApeironTerm::to_canonical_string(&parsed_terms);
                let coord = ApeironCoord::from_str_canonical(&canon);
                items.push(ExtractedItem {
                    coord,
                    key: Some(key.to_string()),
                    terms: parsed_terms,
                    canonical_text: canon,
                    context: Some(format!("iOS Strings in {file_path}:{}", idx + 1)),
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

/// Extracts translatable entries from Java / Kotlin `.properties` files.
pub fn extract_properties(source: &str, file_path: &str) -> Vec<ExtractedItem> {
    let mut items = Vec::new();

    for (idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('!') {
            continue;
        }

        let delimiter = if let Some(pos) = trimmed.find('=') {
            Some((pos, '='))
        } else {
            trimmed.find(':').map(|pos| (pos, ':'))
        };

        if let Some((pos, _)) = delimiter {
            let key = trimmed[..pos].trim();
            let val = trimmed[pos + 1..].trim();

            if !val.is_empty() {
                let parsed_terms = parse_interpolated_string(val);
                let canon = ApeironTerm::to_canonical_string(&parsed_terms);
                let coord = ApeironCoord::from_str_canonical(&canon);
                items.push(ExtractedItem {
                    coord,
                    key: Some(key.to_string()),
                    terms: parsed_terms,
                    canonical_text: canon,
                    context: Some(format!("Properties in {file_path}:{}", idx + 1)),
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

/// Extracts translatable units from XLIFF 1.2 and 2.0 XML files (.xlf, .xliff).
pub fn extract_xliff(source: &str, file_path: &str) -> Vec<ExtractedItem> {
    let mut items = Vec::new();
    let mut cursor = 0;

    // Fast parser for <trans-unit id="..."> ... <source>...</source>
    while let Some(unit_start) = source[cursor..].find("<trans-unit") {
        let actual_start = cursor + unit_start;
        if let Some(unit_end) = source[actual_start..].find("</trans-unit>") {
            let unit_block = &source[actual_start..actual_start + unit_end + 13];
            let id = extract_attribute(&unit_block[..unit_block.find('>').unwrap_or(unit_block.len())], "id");

            if let Some(src_start) = unit_block.find("<source>") {
                let content_start = src_start + 8;
                if let Some(src_end) = unit_block[content_start..].find("</source>") {
                    let source_text = unit_block[content_start..content_start + src_end].trim();
                    if !source_text.is_empty() {
                        let parsed_terms = parse_interpolated_string(source_text);
                        let canon = ApeironTerm::to_canonical_string(&parsed_terms);
                        let coord = ApeironCoord::from_str_canonical(&canon);
                        items.push(ExtractedItem {
                            coord,
                            key: id,
                            terms: parsed_terms,
                            canonical_text: canon,
                            context: Some(format!("XLIFF in {file_path}")),
                            locations: vec![SourceLocation {
                                file_path: file_path.to_string(),
                                line: 1,
                                column: 1,
                            }],
                        });
                    }
                }
            }
            cursor = actual_start + unit_end + 13;
            continue;
        }
        cursor = actual_start + 11;
    }

    items
}

/// Extracts translatable entries from CSV or TSV files.
pub fn extract_csv(source: &str, file_path: &str) -> Vec<ExtractedItem> {
    let mut items = Vec::new();

    for (idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || (idx == 0 && (trimmed.starts_with("key,") || trimmed.starts_with("id,"))) {
            continue;
        }

        let sep = if trimmed.contains('\t') { '\t' } else { ',' };
        if let Some((key, val)) = trimmed.split_once(sep) {
            let clean_key = key.trim().trim_matches('"');
            let clean_val = val.trim().trim_matches('"');

            if !clean_val.is_empty() {
                let parsed_terms = parse_interpolated_string(clean_val);
                let canon = ApeironTerm::to_canonical_string(&parsed_terms);
                let coord = ApeironCoord::from_str_canonical(&canon);
                items.push(ExtractedItem {
                    coord,
                    key: Some(clean_key.to_string()),
                    terms: parsed_terms,
                    canonical_text: canon,
                    context: Some(format!("CSV in {file_path}:{}", idx + 1)),
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

/// Helper to extract attribute values from XML tag headers: e.g. name="foo" -> Some("foo")
fn extract_attribute(tag: &str, attr: &str) -> Option<String> {
    let pattern = format!("{attr}=\"");
    if let Some(pos) = tag.find(&pattern) {
        let start = pos + pattern.len();
        if let Some(end) = tag[start..].find('"') {
            return Some(tag[start..start + end].to_string());
        }
    }
    None
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
    let replaced = s.replace("{$", "{");
    parse_interpolated_string(&replaced)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_yaml() {
        let yaml = r#"
app:
  title: "Dashboard Overview"
  nav:
    settings: Settings
    profile: User Profile
"#;
        let items = extract_yaml(yaml, "locales/en.yml");
        assert_eq!(items.len(), 3);
        assert!(items.iter().any(|i| i.key.as_deref() == Some("app.title")));
        assert!(items.iter().any(|i| i.canonical_text == "Dashboard Overview"));
    }

    #[test]
    fn test_extract_android_xml() {
        let xml = r#"
<resources>
    <string name="app_name">My Super App</string>
    <string name="welcome_message">Welcome back, {user}!</string>
</resources>
"#;
        let items = extract_android_xml(xml, "values/strings.xml");
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|i| i.key.as_deref() == Some("app_name")));
        assert!(items.iter().any(|i| i.canonical_text == "Welcome back, {user}!"));
    }

    #[test]
    fn test_extract_ios_strings() {
        let strings = r#"
/* Header comment */
"login_button" = "Log In";
"items_badge" = "You have %@ new messages";
"#;
        let items = extract_ios_strings(strings, "Localizable.strings");
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|i| i.key.as_deref() == Some("login_button")));
        assert!(items.iter().any(|i| i.canonical_text == "You have {var} new messages"));
    }

    #[test]
    fn test_extract_flutter_arb() {
        let arb = r#"{
            "helloWorld": "Hello World!",
            "@helloWorld": {
                "description": "The conventional newborn programmer greeting"
            },
            "welcomeUser": "Welcome, {username}!"
        }"#;
        let items = extract_flutter_arb(arb, "intl_en.arb");
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|i| i.canonical_text == "Hello World!"));
        assert!(items.iter().any(|i| i.canonical_text == "Welcome, {username}!"));
    }

    #[test]
    fn test_extract_properties() {
        let prop = r#"
# Database config strings
prompt.retry = Please try again later.
prompt.success: Operation succeeded!
"#;
        let items = extract_properties(prop, "messages.properties");
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|i| i.canonical_text == "Please try again later."));
    }

    #[test]
    fn test_extract_xliff() {
        let xlf = r#"
<xliff version="1.2">
  <file>
    <body>
      <trans-unit id="msg_1">
        <source>Hello from XLIFF</source>
        <target></target>
      </trans-unit>
    </body>
  </file>
</xliff>
"#;
        let items = extract_xliff(xlf, "messages.xlf");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].canonical_text, "Hello from XLIFF");
    }

    #[test]
    fn test_extract_csv() {
        let csv = "key,text\nwelcome,Welcome to our website\nbye,Goodbye!\n";
        let items = extract_csv(csv, "catalog.csv");
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|i| i.canonical_text == "Welcome to our website"));
    }
}
