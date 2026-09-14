//! High-Speed JSX/TSX Parser & Extractor (React, Preact, Solid).
//!
//! Lifts JSX AST constructs directly into canonical ApeironTerm nodes
//! with zero external Node.js/Babel dependencies.

use crate::ir::coord::ApeironCoord;
use crate::ir::term::{normalize_whitespace, ApeironTerm, ExtractedItem, SourceLocation};

/// Translatable attribute names in JSX/HTML.
const TRANSLATABLE_ATTRS: &[&str] = &[
    "placeholder",
    "title",
    "alt",
    "aria-label",
    "aria-placeholder",
    "aria-roledescription",
    "label",
];

/// Extracts translatable units from JSX/TSX source code.
pub fn extract_jsx(source: &str, file_path: &str) -> Vec<ExtractedItem> {
    let mut items = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut idx = 0;

    let mut line = 1;
    let mut col = 1;

    while idx < len {
        // Track line/column
        if chars[idx] == '\n' {
            line += 1;
            col = 1;
            idx += 1;
            continue;
        }

        // Check for JSX opening tag `<Tag` (not `</`, not `<!--`, not `<!`, not `<?`)
        if chars[idx] == '<' && idx + 1 < len {
            let next = chars[idx + 1];
            if next.is_ascii_alphabetic() {
                // Parse element
                let tag_start_line = line;
                let tag_start_col = col;
                if let Some((new_idx, elem_terms, attr_items)) =
                    parse_jsx_element(&chars, idx, &mut line, &mut col, file_path)
                {
                    // Add attribute items
                    items.extend(attr_items);

                    // If the element has meaningful text terms, add as translatable item
                    let normalized = ApeironTerm::normalize_sequence(elem_terms);
                    if has_translatable_content(&normalized) {
                        let canon = ApeironTerm::to_canonical_string(&normalized);
                        let coord = ApeironCoord::from_str_canonical(&canon);
                        items.push(ExtractedItem {
                            coord,
                            key: None,
                            terms: normalized,
                            canonical_text: canon,
                            context: Some(format!("JSX in {file_path}:{tag_start_line}")),
                            locations: vec![SourceLocation {
                                file_path: file_path.to_string(),
                                line: tag_start_line,
                                column: tag_start_col,
                            }],
                        });
                    }

                    idx = new_idx;
                    continue;
                }
            }
        }

        col += 1;
        idx += 1;
    }

    items
}

/// Checks if a list of terms contains actual translatable text (not just whitespace or empty).
fn has_translatable_content(terms: &[ApeironTerm]) -> bool {
    terms.iter().any(|t| match t {
        ApeironTerm::Text(s) => s.chars().any(|c| c.is_alphanumeric()),
        ApeironTerm::Tag { children, .. } => has_translatable_content(children),
        ApeironTerm::Plural { .. } | ApeironTerm::Select { .. } => true,
        ApeironTerm::Slot { .. } => false,
    })
}

/// Parse a JSX element starting at '<'.
/// Returns (new_index, child_terms, extracted_attribute_items)
fn parse_jsx_element(
    chars: &[char],
    start: usize,
    line: &mut usize,
    col: &mut usize,
    file_path: &str,
) -> Option<(usize, Vec<ApeironTerm>, Vec<ExtractedItem>)> {
    let mut i = start + 1; // skip '<'
    let mut tag_name = String::new();

    // Read tag name
    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '.' || chars[i] == '-' || chars[i] == '_') {
        tag_name.push(chars[i]);
        i += 1;
    }

    if tag_name.is_empty() {
        return None;
    }

    // Skip non-translatable tags
    let is_ignored = tag_name == "script" || tag_name == "style" || tag_name == "code" || tag_name == "pre";

    let mut attr_items = Vec::new();
    let mut is_self_closing = false;

    // Parse attributes until '>' or '/>'
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

        // Read attribute name
        let attr_line = *line;
        let attr_col = *col;
        let mut attr_name = String::new();
        while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '-' || chars[i] == '_') {
            attr_name.push(chars[i]);
            i += 1;
        }

        skip_ws(chars, &mut i, line, col);
        if i < chars.len() && chars[i] == '=' {
            i += 1; // skip '='
            skip_ws(chars, &mut i, line, col);
            if i < chars.len() {
                if chars[i] == '"' || chars[i] == '\'' {
                    let quote = chars[i];
                    i += 1;
                    let mut attr_val = String::new();
                    while i < chars.len() && chars[i] != quote {
                        if chars[i] == '\\' && i + 1 < chars.len() {
                            i += 1;
                        }
                        attr_val.push(chars[i]);
                        i += 1;
                    }
                    if i < chars.len() {
                        i += 1; // skip closing quote
                    }

                    // If attribute is translatable
                    if TRANSLATABLE_ATTRS.contains(&attr_name.as_str()) && !attr_val.trim().is_empty() {
                        let trimmed = attr_val.trim().to_string();
                        let canon = trimmed.clone();
                        let coord = ApeironCoord::from_str_canonical(&canon);
                        attr_items.push(ExtractedItem {
                            coord,
                            key: None,
                            terms: vec![ApeironTerm::Text(trimmed.clone())],
                            canonical_text: canon,
                            context: Some(format!("Attribute {attr_name} in {tag_name}")),
                            locations: vec![SourceLocation {
                                file_path: file_path.to_string(),
                                line: attr_line,
                                column: attr_col,
                            }],
                        });
                    }
                } else if chars[i] == '{' {
                    // JSX expression attribute, skip to matching '}'
                    let _ = skip_curly(chars, &mut i, line, col);
                }
            }
        }
    }

    if is_self_closing || is_ignored {
        return Some((i, Vec::new(), attr_items));
    }

    // Now parse children until matching closing tag `</tag_name>`
    let mut children = Vec::new();
    let mut slot_counter = 0;
    let mut tag_counter = 0;

    while i < chars.len() {
        // Check for closing tag of this element
        if chars[i] == '<' && i + 1 < chars.len() && chars[i + 1] == '/' {
            let mut close_idx = i + 2;
            let mut close_tag = String::new();
            while close_idx < chars.len() && (chars[close_idx].is_alphanumeric() || chars[close_idx] == '.' || chars[close_idx] == '-' || chars[close_idx] == '_') {
                close_tag.push(chars[close_idx]);
                close_idx += 1;
            }
            skip_ws(chars, &mut close_idx, line, col);
            if close_idx < chars.len() && chars[close_idx] == '>' && close_tag == tag_name {
                i = close_idx + 1;
                break;
            }
        }

        // Child JSX element
        if chars[i] == '<' && i + 1 < chars.len() && chars[i + 1].is_ascii_alphabetic() {
            if let Some((new_idx, sub_terms, sub_attrs)) =
                parse_jsx_element(chars, i, line, col, file_path)
            {
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

        // JSX expression child `{expr}`
        if chars[i] == '{' {
            let expr_start = i + 1;
            if let Some(expr_end) = skip_curly(chars, &mut i, line, col) {
                let expr_str: String = chars[expr_start..expr_end].iter().collect();
                let trimmed = expr_str.trim();
                let name = sanitize_slot_name(trimmed, slot_counter);
                children.push(ApeironTerm::Slot {
                    id: slot_counter,
                    name,
                    raw_expr: Some(trimmed.to_string()),
                });
                slot_counter += 1;
                continue;
            }
        }

        // Plain text character
        let mut text = String::new();
        while i < chars.len() && chars[i] != '<' && chars[i] != '{' {
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

fn skip_curly(chars: &[char], i: &mut usize, line: &mut usize, col: &mut usize) -> Option<usize> {
    if *i >= chars.len() || chars[*i] != '{' {
        return None;
    }
    *i += 1;
    let mut depth = 1;
    while *i < chars.len() && depth > 0 {
        if chars[*i] == '{' {
            depth += 1;
        } else if chars[*i] == '}' {
            depth -= 1;
            if depth == 0 {
                let end = *i;
                *i += 1;
                return Some(end);
            }
        } else if chars[*i] == '\n' {
            *line += 1;
            *col = 1;
        } else {
            *col += 1;
        }
        *i += 1;
    }
    None
}

/// Creates a safe identifier for slot from expression.
fn sanitize_slot_name(expr: &str, fallback_id: usize) -> String {
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
    fn test_extract_jsx_basic() {
        let jsx = r#"
        export default function Welcome({ name, count }) {
            return (
                <div className="card">
                    <h1>Welcome to our platform</h1>
                    <p>Hello {name}, you have {count} notifications.</p>
                    <button placeholder="Enter email">Click <span className="bold">here</span> to continue</button>
                </div>
            );
        }
        "#;

        let items = extract_jsx(jsx, "Welcome.tsx");
        assert!(!items.is_empty());
        let texts: Vec<&str> = items.iter().map(|it| it.canonical_text.as_str()).collect();
        assert!(texts.iter().any(|t| t.contains("Welcome to our platform")));
        assert!(texts.iter().any(|t| t.contains("Hello {name}, you have {count} notifications.")));
        assert!(texts.iter().any(|t| t.contains("Click <0>here</0> to continue")));
        assert!(texts.iter().any(|t| t.contains("Enter email")));
    }
}
