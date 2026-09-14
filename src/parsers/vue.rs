//! High-Speed Vue SFC Parser & Extractor (.vue).
//!
//! Extracts translatable strings and slots from Vue templates,
//! converting Mustache interpolations `{{ user }}` into canonical ApeironTerm slots.

use crate::ir::coord::ApeironCoord;
use crate::ir::term::{normalize_whitespace, ApeironTerm, ExtractedItem, SourceLocation};

pub fn extract_vue(source: &str, file_path: &str) -> Vec<ExtractedItem> {
    let mut items = Vec::new();

    // Find `<template>` block
    let lower = source.to_ascii_lowercase();
    let template_start = match lower.find("<template") {
        Some(idx) => {
            // Find end of `<template...>` tag
            match source[idx..].find('>') {
                Some(gt) => idx + gt + 1,
                None => return items,
            }
        }
        None => return items,
    };

    let template_end = match lower.rfind("</template>") {
        Some(idx) if idx > template_start => idx,
        _ => source.len(),
    };

    let template_content = &source[template_start..template_end];
    let template_line_offset = source[..template_start].matches('\n').count();

    let chars: Vec<char> = template_content.chars().collect();
    let len = chars.len();
    let mut idx = 0;
    let mut line = 1 + template_line_offset;
    let mut col = 1;

    while idx < len {
        if chars[idx] == '\n' {
            line += 1;
            col = 1;
            idx += 1;
            continue;
        }

        // Element start
        if chars[idx] == '<' && idx + 1 < len && chars[idx + 1].is_ascii_alphabetic() {
            let elem_line = line;
            let elem_col = col;

            if let Some((new_idx, terms, attrs)) = parse_vue_element(&chars, idx, &mut line, &mut col, file_path) {
                items.extend(attrs);

                let normalized = ApeironTerm::normalize_sequence(terms);
                if has_translatable_content(&normalized) {
                    let canon = ApeironTerm::to_canonical_string(&normalized);
                    let coord = ApeironCoord::from_str_canonical(&canon);
                    items.push(ExtractedItem {
                        coord,
                        key: None,
                        terms: normalized,
                        canonical_text: canon,
                        context: Some(format!("Vue template in {file_path}:{elem_line}")),
                        locations: vec![SourceLocation {
                            file_path: file_path.to_string(),
                            line: elem_line,
                            column: elem_col,
                        }],
                    });
                }
                idx = new_idx;
                continue;
            }
        }

        col += 1;
        idx += 1;
    }

    items
}

fn has_translatable_content(terms: &[ApeironTerm]) -> bool {
    terms.iter().any(|t| match t {
        ApeironTerm::Text(s) => s.chars().any(|c| c.is_alphanumeric()),
        ApeironTerm::Tag { children, .. } => has_translatable_content(children),
        ApeironTerm::Plural { .. } | ApeironTerm::Select { .. } => true,
        ApeironTerm::Slot { .. } => false,
    })
}

fn parse_vue_element(
    chars: &[char],
    start: usize,
    line: &mut usize,
    col: &mut usize,
    file_path: &str,
) -> Option<(usize, Vec<ApeironTerm>, Vec<ExtractedItem>)> {
    let mut i = start + 1;
    let mut tag_name = String::new();

    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '-' || chars[i] == '_') {
        tag_name.push(chars[i]);
        i += 1;
    }

    if tag_name.is_empty() {
        return None;
    }

    let mut attr_items = Vec::new();
    let mut is_self_closing = false;

    // Parse attributes
    while i < chars.len() {
        skip_ws(chars, &mut i, line, col);
        if i >= chars.len() {
            return None;
        }

        if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '>' {
            i += 2;
            is_self_closing = true;
            break;
        }
        if chars[i] == '>' {
            i += 1;
            break;
        }

        let attr_line = *line;
        let attr_col = *col;
        let mut attr_name = String::new();
        while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '-' || chars[i] == '_' || chars[i] == ':' || chars[i] == '@') {
            attr_name.push(chars[i]);
            i += 1;
        }

        skip_ws(chars, &mut i, line, col);
        if i < chars.len() && chars[i] == '=' {
            i += 1;
            skip_ws(chars, &mut i, line, col);
            if i < chars.len() && (chars[i] == '"' || chars[i] == '\'') {
                let quote = chars[i];
                i += 1;
                let mut attr_val = String::new();
                while i < chars.len() && chars[i] != quote {
                    attr_val.push(chars[i]);
                    i += 1;
                }
                if i < chars.len() {
                    i += 1;
                }

                if (attr_name == "placeholder" || attr_name == "title" || attr_name == "alt" || attr_name == "aria-label")
                    && !attr_val.trim().is_empty()
                {
                    let trimmed = attr_val.trim().to_string();
                    let canon = trimmed.clone();
                    let coord = ApeironCoord::from_str_canonical(&canon);
                    attr_items.push(ExtractedItem {
                        coord,
                        key: None,
                        terms: vec![ApeironTerm::Text(trimmed)],
                        canonical_text: canon,
                        context: Some(format!("Vue attribute {attr_name} in {tag_name}")),
                        locations: vec![SourceLocation {
                            file_path: file_path.to_string(),
                            line: attr_line,
                            column: attr_col,
                        }],
                    });
                }
            }
        }
    }

    if is_self_closing || tag_name == "script" || tag_name == "style" {
        return Some((i, Vec::new(), attr_items));
    }

    let mut children = Vec::new();
    let mut slot_counter = 0;
    let mut tag_counter = 0;

    while i < chars.len() {
        // Closing tag check
        if chars[i] == '<' && i + 1 < chars.len() && chars[i + 1] == '/' {
            let mut close_idx = i + 2;
            let mut close_tag = String::new();
            while close_idx < chars.len() && (chars[close_idx].is_alphanumeric() || chars[close_idx] == '-' || chars[close_idx] == '_') {
                close_tag.push(chars[close_idx]);
                close_idx += 1;
            }
            skip_ws(chars, &mut close_idx, line, col);
            if close_idx < chars.len() && chars[close_idx] == '>' && close_tag == tag_name {
                i = close_idx + 1;
                break;
            }
        }

        // Child element
        if chars[i] == '<' && i + 1 < chars.len() && chars[i + 1].is_ascii_alphabetic() {
            if let Some((new_idx, sub_terms, sub_attrs)) = parse_vue_element(chars, i, line, col, file_path) {
                attr_items.extend(sub_attrs);
                let tag_id = tag_counter;
                tag_counter += 1;
                children.push(ApeironTerm::Tag {
                    id: tag_id,
                    tag: "tag".to_string(),
                    attrs: Vec::new(),
                    children: sub_terms,
                });
                i = new_idx;
                continue;
            }
        }

        // Vue Mustache interpolation `{{ expr }}`
        if chars[i] == '{' && i + 1 < chars.len() && chars[i + 1] == '{' {
            i += 2;
            let mut expr = String::new();
            while i + 1 < chars.len() && !(chars[i] == '}' && chars[i + 1] == '}') {
                expr.push(chars[i]);
                i += 1;
            }
            if i + 1 < chars.len() {
                i += 2; // skip '}}'
            }
            let trimmed = expr.trim();
            let name = sanitize_name(trimmed, slot_counter);
            children.push(ApeironTerm::Slot {
                id: slot_counter,
                name,
                raw_expr: Some(trimmed.to_string()),
            });
            slot_counter += 1;
            continue;
        }

        // Plain text
        let mut text = String::new();
        while i < chars.len() && chars[i] != '<' && !(chars[i] == '{' && i + 1 < chars.len() && chars[i + 1] == '{') {
            if chars[i] == '\n' {
                *line += 1;
                *col = 1;
            } else {
                *col += 1;
            }
            text.push(chars[i]);
            i += 1;
        }

        let norm = normalize_whitespace(&text);
        if !norm.trim().is_empty() {
            children.push(ApeironTerm::Text(norm));
        }
    }

    Some((i, children, attr_items))
}

fn skip_ws(chars: &[char], i: &mut usize, line: &mut usize, col: &mut usize) {
    while *i < chars.len() && chars[*i].is_whitespace() {
        if chars[*i] == '\n' {
            *line += 1;
            *col = 1;
        } else {
            *col += 1;
        }
        *i += 1;
    }
}

fn sanitize_name(expr: &str, fallback_id: usize) -> String {
    let clean: String = expr
        .chars()
        .map(|c| if c == '.' { '_' } else { c })
        .filter(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if clean.is_empty() || clean.chars().next().unwrap().is_numeric() {
        format!("var{fallback_id}")
    } else if clean.len() > 20 {
        format!("var{fallback_id}")
    } else {
        clean
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_vue_template() {
        let vue = r#"
        <template>
            <div class="user-profile">
                <h2>User Profile</h2>
                <p>Welcome back, {{ user.name }}!</p>
                <button placeholder="Search profiles">Visit <a href="/docs">docs</a></button>
            </div>
        </template>
        <script>
        export default { name: 'UserProfile' }
        </script>
        "#;

        let items = extract_vue(vue, "UserProfile.vue");
        assert!(!items.is_empty());
        let texts: Vec<&str> = items.iter().map(|it| it.canonical_text.as_str()).collect();
        assert!(texts.iter().any(|t| t.contains("User Profile")));
        assert!(texts.iter().any(|t| t.contains("Welcome back, {user_name}!")));
        assert!(texts.iter().any(|t| t.contains("Visit <0>docs</0>")));
        assert!(texts.iter().any(|t| t.contains("Search profiles")));
    }
}
