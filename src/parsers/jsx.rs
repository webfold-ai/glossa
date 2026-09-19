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

#[derive(Clone, Debug)]
enum JsxNode {
    Text(String),
    Slot(String),
    Element {
        tag: String,
        line: usize,
        col: usize,
        attrs: Vec<(String, String)>,
        children: Vec<JsxNode>,
    },
}

/// Checks if a tag is a structural container rather than a single phrase/message.
fn is_container_tag(tag: &str) -> bool {
    if tag.chars().next().map_or(false, |c| c.is_ascii_uppercase()) {
        return true;
    }
    matches!(
        tag.to_ascii_lowercase().as_str(),
        "div"
            | "section"
            | "header"
            | "footer"
            | "main"
            | "aside"
            | "dialog"
            | "nav"
            | "form"
            | "body"
            | "html"
            | "template"
            | "ul"
            | "ol"
            | "table"
            | "thead"
            | "tbody"
            | "tr"
            | "fieldset"
            | "article"
    )
}

/// Extracts translatable units from JSX/TSX source code.
pub fn extract_jsx(source: &str, file_path: &str) -> Vec<ExtractedItem> {
    let mut items = Vec::new();
    let mut seen_coords = std::collections::BTreeSet::new();

    // 1. Extract explicit code translation calls (t('...'), formatMessage, etc.) in the file
    let code_items = crate::parsers::code::extract_code_strings(source, file_path);
    for item in code_items {
        if seen_coords.insert(item.coord) {
            items.push(item);
        }
    }

    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut idx = 0;

    let mut line = 1;
    let mut col = 1;

    while idx < len {
        if chars[idx] == '\n' {
            line += 1;
            col = 1;
            idx += 1;
            continue;
        }

        if chars[idx] == '<' && idx + 1 < len {
            let next = chars[idx + 1];
            if next.is_ascii_alphabetic() {
                if let Some((new_idx, node)) = parse_jsx_element_node(&chars, idx, &mut line, &mut col) {
                    collect_jsx_items(&node, file_path, &mut items, &mut seen_coords);
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

fn collect_jsx_items(
    node: &JsxNode,
    file_path: &str,
    items: &mut Vec<ExtractedItem>,
    seen_coords: &mut std::collections::BTreeSet<ApeironCoord>,
) {
    if let JsxNode::Element {
        tag,
        line,
        col,
        attrs,
        children,
    } = node
    {
        // 1. Collect translatable attributes
        for (attr_name, attr_val) in attrs {
            if TRANSLATABLE_ATTRS.contains(&attr_name.as_str()) && !attr_val.trim().is_empty() {
                let clean_val = crate::parsers::code::extract_literal_from_call(attr_val)
                    .unwrap_or_else(|| attr_val.trim().to_string());
                if !clean_val.trim().is_empty() {
                    let canon = clean_val.clone();
                    let coord = ApeironCoord::from_str_canonical(&canon);
                    if seen_coords.insert(coord) {
                        items.push(ExtractedItem {
                            coord,
                            key: None,
                            terms: vec![ApeironTerm::Text(clean_val)],
                            canonical_text: canon,
                            context: Some(format!("Attribute {attr_name} in <{tag}>")),
                            locations: vec![SourceLocation {
                                file_path: file_path.to_string(),
                                line: *line,
                                column: *col,
                            }],
                        });
                    }
                }
            }
        }

        // 2. Check container semantics
        let has_child_elements = children.iter().any(|c| matches!(c, JsxNode::Element { .. }));

        if is_container_tag(tag) && has_child_elements {
            for child in children {
                match child {
                    JsxNode::Element { .. } => collect_jsx_items(child, file_path, items, seen_coords),
                    JsxNode::Text(t) => {
                        let norm = normalize_whitespace(t);
                        if !norm.trim().is_empty() && norm.chars().any(|c| c.is_alphanumeric()) {
                            let canon = norm.clone();
                            let coord = ApeironCoord::from_str_canonical(&canon);
                            if seen_coords.insert(coord) {
                                items.push(ExtractedItem {
                                    coord,
                                    key: None,
                                    terms: vec![ApeironTerm::Text(norm)],
                                    canonical_text: canon,
                                    context: Some(format!("Text in <{tag}>")),
                                    locations: vec![SourceLocation {
                                        file_path: file_path.to_string(),
                                        line: *line,
                                        column: *col,
                                    }],
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
        } else {
            // Leaf element or non-container: extract terms
            let mut terms = Vec::new();
            let mut tag_counter = 0;
            let mut slot_counter = 0;
            jsx_nodes_to_terms(children, &mut terms, &mut tag_counter, &mut slot_counter);

            let mut normalized = ApeironTerm::normalize_sequence(terms);
            if let Some(ApeironTerm::Text(first)) = normalized.first_mut() {
                *first = first.trim_start().to_string();
            }
            if let Some(ApeironTerm::Text(last)) = normalized.last_mut() {
                *last = last.trim_end().to_string();
            }
            normalized.retain(|t| match t {
                ApeironTerm::Text(s) => !s.is_empty(),
                _ => true,
            });

            if has_translatable_content(&normalized) {
                let canon = ApeironTerm::to_canonical_string(&normalized);
                let coord = ApeironCoord::from_str_canonical(&canon);
                if seen_coords.insert(coord) {
                    items.push(ExtractedItem {
                        coord,
                        key: None,
                        terms: normalized,
                        canonical_text: canon,
                        context: Some(format!("JSX <{tag}> in {file_path}:{line}")),
                        locations: vec![SourceLocation {
                            file_path: file_path.to_string(),
                            line: *line,
                            column: *col,
                        }],
                    });
                }
            }
        }
    }
}

fn jsx_nodes_to_terms(
    nodes: &[JsxNode],
    terms: &mut Vec<ApeironTerm>,
    tag_counter: &mut usize,
    slot_counter: &mut usize,
) {
    for node in nodes {
        match node {
            JsxNode::Text(t) => {
                terms.push(ApeironTerm::Text(t.clone()));
            }
            JsxNode::Slot(name) => {
                terms.push(ApeironTerm::Slot {
                    id: *slot_counter,
                    name: name.clone(),
                    raw_expr: Some(name.clone()),
                });
                *slot_counter += 1;
            }
            JsxNode::Element { tag, children, .. } => {
                let tag_id = *tag_counter;
                *tag_counter += 1;
                let mut sub_terms = Vec::new();
                jsx_nodes_to_terms(children, &mut sub_terms, tag_counter, slot_counter);
                terms.push(ApeironTerm::Tag {
                    id: tag_id,
                    tag: tag.clone(),
                    attrs: Vec::new(),
                    children: sub_terms,
                });
            }
        }
    }
}

fn has_translatable_content(terms: &[ApeironTerm]) -> bool {
    terms.iter().any(|t| match t {
        ApeironTerm::Text(s) => s.chars().any(|c| c.is_alphanumeric()),
        ApeironTerm::Tag { children, .. } => has_translatable_content(children),
        ApeironTerm::Plural { .. } | ApeironTerm::Select { .. } => true,
        ApeironTerm::Slot { .. } => false,
    })
}

fn parse_jsx_element_node(
    chars: &[char],
    start: usize,
    line: &mut usize,
    col: &mut usize,
) -> Option<(usize, JsxNode)> {
    let mut i = start + 1; // skip '<'
    let tag_line = *line;
    let tag_col = *col;
    let mut tag_name = String::new();

    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '.' || chars[i] == '-' || chars[i] == '_') {
        tag_name.push(chars[i]);
        i += 1;
    }

    if tag_name.is_empty() {
        return None;
    }

    let is_ignored = tag_name == "script" || tag_name == "style" || tag_name == "code" || tag_name == "pre";
    let mut attrs = Vec::new();
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

        let mut attr_name = String::new();
        while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '-' || chars[i] == '_') {
            attr_name.push(chars[i]);
            i += 1;
        }

        // A real infinite loop, found by testing against a real production
        // file (gitroomhq/postiz-app's apps/frontend/public/f.js): if the
        // current character starts neither a valid attr name, `/`, `>`, nor
        // is whitespace, NOTHING above advances `i` and this `while i <
        // chars.len()` loop repeats forever at the identical position.
        // Ordinary JS/TS comparisons with no space — `i<len;i++`, `i<len)`,
        // `i<len&&...` — all hit this: after the outer `<` + alphabetic-next
        // check treats `len` as a plausible tag name, the very next
        // character (`;`, `)`, `&`) is neither an attr-name start nor `=`,
        // so parsing never terminates. Any `.js`/`.jsx`/`.tsx` file with a
        // normal loop condition triggers this — this is not a rare edge
        // case, every real-world non-JSX .js file that reaches here hangs
        // the whole extraction forever. If nothing here looks like valid
        // JSX attribute syntax, this was never a real tag — bail out rather
        // than loop.
        if attr_name.is_empty() && (i >= chars.len() || chars[i] != '=') {
            return None;
        }

        skip_ws(chars, &mut i, line, col);
        if i < chars.len() && chars[i] == '=' {
            i += 1;
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
                        i += 1;
                    }
                    attrs.push((attr_name, attr_val));
                } else if chars[i] == '{' {
                    let _ = skip_curly(chars, &mut i, line, col);
                }
            }
        }
    }

    if is_self_closing || is_ignored {
        return Some((
            i,
            JsxNode::Element {
                tag: tag_name,
                line: tag_line,
                col: tag_col,
                attrs,
                children: Vec::new(),
            },
        ));
    }

    // Parse children until matching </tag_name>
    let mut children = Vec::new();
    let mut slot_counter = 0;

    while i < chars.len() {
        // Closing tag check
        if chars[i] == '<' && i + 1 < chars.len() && chars[i + 1] == '/' {
            let mut close_idx = i + 2;
            let mut close_tag = String::new();
            while close_idx < chars.len()
                && (chars[close_idx].is_alphanumeric()
                    || chars[close_idx] == '.'
                    || chars[close_idx] == '-'
                    || chars[close_idx] == '_')
            {
                close_tag.push(chars[close_idx]);
                close_idx += 1;
            }
            skip_ws(chars, &mut close_idx, line, col);
            if close_idx < chars.len() && chars[close_idx] == '>' {
                if close_tag == tag_name {
                    i = close_idx + 1;
                }
                break;
            }
        }

        // Child JSX element
        if chars[i] == '<' && i + 1 < chars.len() && chars[i + 1].is_ascii_alphabetic() {
            if let Some((new_idx, child_node)) = parse_jsx_element_node(chars, i, line, col) {
                children.push(child_node);
                i = new_idx;
                continue;
            }
        }

        // JSX expression child {expr}
        if chars[i] == '{' {
            let expr_start = i + 1;
            if let Some(expr_end) = skip_curly(chars, &mut i, line, col) {
                let expr_str: String = chars[expr_start..expr_end].iter().collect();
                let trimmed = expr_str.trim();
                if let Some(literal) = crate::parsers::code::extract_literal_from_call(trimmed) {
                    children.push(JsxNode::Text(literal));
                } else {
                    let name = sanitize_slot_name(trimmed, slot_counter);
                    children.push(JsxNode::Slot(name));
                    slot_counter += 1;
                }
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
            children.push(JsxNode::Text(norm));
        }
    }

    Some((
        i,
        JsxNode::Element {
            tag: tag_name,
            line: tag_line,
            col: tag_col,
            attrs,
            children,
        },
    ))
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
    let mut in_quote = None;

    while *i < chars.len() && depth > 0 {
        let c = chars[*i];
        if let Some(q) = in_quote {
            if c == '\\' && *i + 1 < chars.len() {
                *i += 1;
            } else if c == q {
                in_quote = None;
            }
        } else {
            if c == '"' || c == '\'' || c == '`' {
                in_quote = Some(c);
            } else if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
                if depth == 0 {
                    let end = *i;
                    *i += 1;
                    return Some(end);
                }
            }
        }
        if c == '\n' {
            *line += 1;
            *col = 1;
        } else {
            *col += 1;
        }
        *i += 1;
    }
    None
}

/// Creates a safe identifier for slot from expression, preserving property dots.
fn sanitize_slot_name(expr: &str, fallback_id: usize) -> String {
    let trimmed = expr.trim();
    let clean: String = trimmed
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '.')
        .collect();
    if clean.is_empty() || clean.starts_with(|c: char| c.is_numeric()) {
        format!("var{fallback_id}")
    } else if clean.len() > 40 {
        format!("var{fallback_id}")
    } else {
        clean
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression guard for a real infinite loop, found by testing
    /// extraction against a real production file
    /// (gitroomhq/postiz-app's apps/frontend/public/f.js, a plain
    /// Facebook-SDK .js file with zero JSX in it, which hung glossa's own
    /// CI benchmark job for 10+ minutes before it was manually canceled).
    /// `.js` files are routed through this JSX extractor unconditionally,
    /// and any ordinary no-space comparison like `i<len` is enough:
    /// `<len` looks like a plausible tag start, then the very next
    /// character (here `;`) matched neither a valid attribute name nor `=`,
    /// so nothing advanced `i` and the attribute-parsing loop spun forever.
    /// This test would hang the entire test suite before the fix — its
    /// mere ability to complete is the assertion.
    #[test]
    fn test_regression_no_space_comparison_does_not_infinite_loop() {
        let source = r#"
        function findIndex(arr, target) {
            for (var i = 0, len = arr.length; i<len; i++) {
                if (arr[i]===target) return i;
            }
            return -1;
        }
        "#;
        let items = extract_jsx(source, "f.js");
        // No real JSX exists in this file; a correct parser extracts nothing
        // (and, most importantly, returns at all).
        assert!(items.is_empty());
    }

    /// Same bug family, different trigger characters commonly found in real
    /// minified/compact JS: `)`, `&&`, `+`.
    #[test]
    fn test_regression_no_space_comparison_variants_do_not_infinite_loop() {
        for source in [
            "if(x<y){doThing();}",
            "while(count<total&&!done){count++;}",
            "const z=a<b+1?a:b;",
        ] {
            let items = extract_jsx(source, "test.js");
            assert!(items.is_empty(), "source `{source}` must extract nothing, not hang");
        }
    }

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
