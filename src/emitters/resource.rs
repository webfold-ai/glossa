//! Resource Catalog Emitter (JSON, PO, Fluent, YAML).
//!
//! Emits translated Apeiron catalogs into standardized i18n file formats.

use std::collections::BTreeMap;

/// Emits translations as formatted JSON.
pub fn emit_json(catalog: &BTreeMap<String, String>) -> String {
    let mut map = serde_json::Map::new();
    for (k, v) in catalog {
        map.insert(k.clone(), serde_json::Value::String(v.clone()));
    }
    serde_json::to_string_pretty(&serde_json::Value::Object(map)).unwrap_or_else(|_| "{}".to_string())
}

/// Emits translations as GNU gettext PO file.
pub fn emit_po(catalog: &BTreeMap<String, String>, locale: &str) -> String {
    let mut out = String::new();
    out.push_str("msgid \"\"\n");
    out.push_str("msgstr \"\"\n");
    out.push_str(&format!("\"Language: {locale}\\n\"\n"));
    out.push_str("\"MIME-Version: 1.0\\n\"\n");
    out.push_str("\"Content-Type: text/plain; charset=UTF-8\\n\"\n");
    out.push_str("\"Content-Transfer-Encoding: 8bit\\n\"\n\n");

    for (k, v) in catalog {
        let escaped_k = escape_po_string(k);
        let escaped_v = escape_po_string(v);
        out.push_str(&format!("msgid \"{escaped_k}\"\n"));
        out.push_str(&format!("msgstr \"{escaped_v}\"\n\n"));
    }

    out
}

/// Emits translations as Mozilla Fluent (.ftl) file.
pub fn emit_fluent(catalog: &BTreeMap<String, String>) -> String {
    let mut out = String::new();
    for (k, v) in catalog {
        let safe_key = k
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect::<String>();
        // Fluent variables {$var}
        let fluent_val = convert_slots_to_fluent(v);
        out.push_str(&format!("{safe_key} = {fluent_val}\n"));
    }
    out
}

fn escape_po_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}

fn convert_slots_to_fluent(s: &str) -> String {
    // Converts `{var}` to `{$var}`
    let mut out = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' && i + 1 < chars.len() && chars[i + 1] != '{' && chars[i + 1] != '$' {
            out.push('{');
            out.push('$');
            i += 1;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_formats() {
        let mut map = BTreeMap::new();
        map.insert("hello".to_string(), "Hello, {name}!".to_string());

        let json = emit_json(&map);
        assert!(json.contains("\"hello\": \"Hello, {name}!\""));

        let po = emit_po(&map, "es");
        assert!(po.contains("msgid \"hello\""));
        assert!(po.contains("msgstr \"Hello, {name}!\""));

        let ftl = emit_fluent(&map);
        assert_eq!(ftl, "hello = Hello, {$name}!\n");
    }
}
