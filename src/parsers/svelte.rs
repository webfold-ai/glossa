//! High-Speed Svelte Parser & Extractor (.svelte).
//!
//! Extracts translatable strings from Svelte templates,
//! handling `{ expression }` blocks, HTML markup, and attributes.

use crate::ir::coord::ApeironCoord;
use crate::ir::term::{normalize_whitespace, ApeironTerm, ExtractedItem, SourceLocation};

const TRANSLATABLE_ATTRS: &[&str] = &[
    "placeholder",
    "title",
    "alt",
    "aria-label",
    "aria-placeholder",
    "label",
];

#[derive(Clone, Debug)]
enum SvelteNode {
    Text(String),
    Slot(String),
    Element {
        tag: String,
        line: usize,
        col: usize,
        attrs: Vec<(String, String)>,
        children: Vec<SvelteNode>,
    },
}

fn is_container_tag(tag: &str) -> bool {
    if tag.chars().next().map_or(false, |c| c.is_ascii_uppercase()) {
        return true;
    }
    matches!(
        tag.to_ascii_lowercase().as_str(),
        "aside"
            | "section"
            | "header"
            | "footer"
            | "main"
            | "nav"
            | "div"
            | "form"
            | "dialog"
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

pub fn extract_svelte(source: &str, file_path: &str) -> Vec<ExtractedItem> {
    let mut items = Vec::new();
    let mut seen_coords = std::collections::BTreeSet::new();

    // 1. Extract script translation calls (t('...'), etc.)
    let code_items = crate::parsers::code::extract_code_strings(source, file_path);
    for item in code_items {
        if seen_coords.insert(item.coord) {
            items.push(item);
        }
    }

    let cleaned = strip_blocks(source, &["script", "style"]);

    let chars: Vec<char> = cleaned.chars().collect();
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

        if chars[idx] == '<' && idx + 1 < len && chars[idx + 1].is_ascii_alphabetic() {
            if let Some((new_idx, node)) = parse_svelte_element_node(&chars, idx, &mut line, &mut col) {
                collect_svelte_items(&node, file_path, &mut items, &mut seen_coords);
                idx = new_idx;
                continue;
            }
        }

        col += 1;
        idx += 1;
    }

    items
}

fn strip_blocks(source: &str, tags: &[&str]) -> String {
    let mut res = source.to_string();
    for tag in tags {
        let open_tag = format!("<{tag}");
        let close_tag = format!("</{tag}>");
        while let Some(start) = res.to_ascii_lowercase().find(&open_tag) {
            if let Some(end_offset) = res.to_ascii_lowercase()[start..].find(&close_tag) {
                let end = start + end_offset + close_tag.len();
                let num_lines = res[start..end].matches('\n').count();
                let replacement = "\n".repeat(num_lines);
                res.replace_range(start..end, &replacement);
            } else {
                break;
            }
        }
    }
    res
}

fn collect_svelte_items(
    node: &SvelteNode,
    file_path: &str,
    items: &mut Vec<ExtractedItem>,
    seen_coords: &mut std::collections::BTreeSet<ApeironCoord>,
) {
    if let SvelteNode::Element {
        tag,
        line,
        col,
        attrs,
        children,
    } = node
    {
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
                            context: Some(format!("Svelte attribute {attr_name} in <{tag}>")),
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

        let has_child_elements = children.iter().any(|c| matches!(c, SvelteNode::Element { .. }));

        if is_container_tag(tag) && has_child_elements {
            for child in children {
                match child {
                    SvelteNode::Element { .. } => collect_svelte_items(child, file_path, items, seen_coords),
                    SvelteNode::Text(t) => {
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
            let mut terms = Vec::new();
            let mut tag_counter = 0;
            let mut slot_counter = 0;
            svelte_nodes_to_terms(children, &mut terms, &mut tag_counter, &mut slot_counter);

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
                        context: Some(format!("Svelte template in {file_path}:{line}")),
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

fn svelte_nodes_to_terms(
    nodes: &[SvelteNode],
    terms: &mut Vec<ApeironTerm>,
    tag_counter: &mut usize,
    slot_counter: &mut usize,
) {
    for node in nodes {
        match node {
            SvelteNode::Text(t) => {
                terms.push(ApeironTerm::Text(t.clone()));
            }
            SvelteNode::Slot(name) => {
                terms.push(ApeironTerm::Slot {
                    id: *slot_counter,
                    name: name.clone(),
                    raw_expr: Some(name.clone()),
                });
                *slot_counter += 1;
            }
            SvelteNode::Element { tag, children, .. } => {
                let tag_id = *tag_counter;
                *tag_counter += 1;
                let mut sub_terms = Vec::new();
                svelte_nodes_to_terms(children, &mut sub_terms, tag_counter, slot_counter);
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

fn parse_svelte_element_node(
    chars: &[char],
    start: usize,
    line: &mut usize,
    col: &mut usize,
) -> Option<(usize, SvelteNode)> {
    let mut i = start + 1;
    let tag_line = *line;
    let tag_col = *col;
    let mut tag_name = String::new();

    while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '-' || chars[i] == '_') {
        tag_name.push(chars[i]);
        i += 1;
    }

    if tag_name.is_empty() {
        return None;
    }

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
        while i < chars.len()
            && (chars[i].is_alphanumeric()
                || chars[i] == '-'
                || chars[i] == '_'
                || chars[i] == ':'
                || chars[i] == '@'
                || chars[i] == '.')
        {
            attr_name.push(chars[i]);
            i += 1;
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
                    // Svelte expression attribute e.g. on:click={() => alert('Saved')}
                    let _ = skip_curly(chars, &mut i, line, col);
                }
            }
        } else if i < chars.len() && chars[i] == '{' {
            // Shorthand attribute {prop}
            let _ = skip_curly(chars, &mut i, line, col);
        } else if attr_name.is_empty() {
            // Guard against infinite loop if unexpected char
            i += 1;
        }
    }

    if is_self_closing {
        return Some((
            i,
            SvelteNode::Element {
                tag: tag_name,
                line: tag_line,
                col: tag_col,
                attrs,
                children: Vec::new(),
            },
        ));
    }

    let mut children = Vec::new();
    let mut slot_counter = 0;

    while i < chars.len() {
        // Check closing tag
        if chars[i] == '<' && i + 1 < chars.len() && chars[i + 1] == '/' {
            let mut close_idx = i + 2;
            let mut close_tag = String::new();
            while close_idx < chars.len()
                && (chars[close_idx].is_alphanumeric() || chars[close_idx] == '-' || chars[close_idx] == '_')
            {
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
            if let Some((new_idx, child_node)) = parse_svelte_element_node(chars, i, line, col) {
                children.push(child_node);
                i = new_idx;
                continue;
            }
        }

        // Svelte expression {expr}
        if chars[i] == '{' {
            let expr_start = i + 1;
            if let Some(expr_end) = skip_curly(chars, &mut i, line, col) {
                let expr_str: String = chars[expr_start..expr_end].iter().collect();
                let trimmed = expr_str.trim();
                if let Some(literal) = crate::parsers::code::extract_literal_from_call(trimmed) {
                    children.push(SvelteNode::Text(literal));
                } else {
                    let name = sanitize_name(trimmed, slot_counter);
                    children.push(SvelteNode::Slot(name));
                    slot_counter += 1;
                }
                continue;
            }
        }

        // Plain text
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
            children.push(SvelteNode::Text(norm));
        }

        // Break on closing tag for parent
        if i < chars.len() && chars[i] == '<' {
            if i + 1 < chars.len() && chars[i + 1] == '/' {
                break;
            }
        }
    }

    Some((
        i,
        SvelteNode::Element {
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

fn sanitize_name(expr: &str, fallback_id: usize) -> String {
    let clean: String = expr
        .trim()
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

    #[test]
    fn test_extract_svelte() {
        let svelte = r#"
        <script>
            export let name = 'World';
            export let unread = 5;
        </script>
        <section>
            <h1>Dashboard</h1>
            <p>Welcome back {name}, you have {unread} messages!</p>
        </section>
        "#;

        let items = extract_svelte(svelte, "Dashboard.svelte");
        assert!(!items.is_empty());
        let texts: Vec<&str> = items.iter().map(|it| it.canonical_text.as_str()).collect();
        assert!(texts.iter().any(|t| t.contains("Dashboard")));
        assert!(texts.iter().any(|t| t.contains("Welcome back {name}, you have {unread} messages!")));
    }
}
