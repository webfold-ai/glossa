//! High-Speed Vue SFC Parser & Extractor (.vue).
//!
//! Extracts translatable strings and slots from Vue templates,
//! converting Mustache interpolations `{{ user }}` into canonical ApeironTerm slots.

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
enum VueNode {
    Text(String),
    Slot(String),
    Element {
        tag: String,
        line: usize,
        col: usize,
        attrs: Vec<(String, String)>,
        children: Vec<VueNode>,
    },
}

fn is_container_tag(tag: &str) -> bool {
    if tag.chars().next().map_or(false, |c| c.is_ascii_uppercase()) {
        return true;
    }
    matches!(
        tag.to_ascii_lowercase().as_str(),
        "section"
            | "header"
            | "footer"
            | "main"
            | "aside"
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

pub fn extract_vue(source: &str, file_path: &str) -> Vec<ExtractedItem> {
    let mut items = Vec::new();

    let lower = source.to_ascii_lowercase();
    let template_start = match lower.find("<template") {
        Some(idx) => match source[idx..].find('>') {
            Some(gt) => idx + gt + 1,
            None => return items,
        },
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

        if chars[idx] == '<' && idx + 1 < len && chars[idx + 1].is_ascii_alphabetic() {
            if let Some((new_idx, node)) = parse_vue_element_node(&chars, idx, &mut line, &mut col) {
                collect_vue_items(&node, file_path, &mut items);
                idx = new_idx;
                continue;
            }
        }

        col += 1;
        idx += 1;
    }

    items
}

fn collect_vue_items(node: &VueNode, file_path: &str, items: &mut Vec<ExtractedItem>) {
    if let VueNode::Element {
        tag,
        line,
        col,
        attrs,
        children,
    } = node
    {
        for (attr_name, attr_val) in attrs {
            if TRANSLATABLE_ATTRS.contains(&attr_name.as_str()) && !attr_val.trim().is_empty() {
                let trimmed = attr_val.trim().to_string();
                let canon = trimmed.clone();
                let coord = ApeironCoord::from_str_canonical(&canon);
                items.push(ExtractedItem {
                    coord,
                    key: None,
                    terms: vec![ApeironTerm::Text(trimmed)],
                    canonical_text: canon,
                    context: Some(format!("Vue attribute {attr_name} in <{tag}>")),
                    locations: vec![SourceLocation {
                        file_path: file_path.to_string(),
                        line: *line,
                        column: *col,
                    }],
                });
            }
        }

        let has_child_elements = children.iter().any(|c| matches!(c, VueNode::Element { .. }));

        if is_container_tag(tag) && has_child_elements {
            for child in children {
                match child {
                    VueNode::Element { .. } => collect_vue_items(child, file_path, items),
                    VueNode::Text(t) => {
                        let norm = normalize_whitespace(t);
                        if !norm.trim().is_empty() && norm.chars().any(|c| c.is_alphanumeric()) {
                            let canon = norm.clone();
                            let coord = ApeironCoord::from_str_canonical(&canon);
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
                    _ => {}
                }
            }
        } else {
            let mut terms = Vec::new();
            let mut tag_counter = 0;
            let mut slot_counter = 0;
            vue_nodes_to_terms(children, &mut terms, &mut tag_counter, &mut slot_counter);

            let normalized = ApeironTerm::normalize_sequence(terms);
            if has_translatable_content(&normalized) {
                let canon = ApeironTerm::to_canonical_string(&normalized);
                let coord = ApeironCoord::from_str_canonical(&canon);
                items.push(ExtractedItem {
                    coord,
                    key: None,
                    terms: normalized,
                    canonical_text: canon,
                    context: Some(format!("Vue template in {file_path}:{line}")),
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

fn vue_nodes_to_terms(
    nodes: &[VueNode],
    terms: &mut Vec<ApeironTerm>,
    tag_counter: &mut usize,
    slot_counter: &mut usize,
) {
    for node in nodes {
        match node {
            VueNode::Text(t) => {
                terms.push(ApeironTerm::Text(t.clone()));
            }
            VueNode::Slot(name) => {
                terms.push(ApeironTerm::Slot {
                    id: *slot_counter,
                    name: name.clone(),
                    raw_expr: Some(name.clone()),
                });
                *slot_counter += 1;
            }
            VueNode::Element { tag, children, .. } => {
                let tag_id = *tag_counter;
                *tag_counter += 1;
                let mut sub_terms = Vec::new();
                vue_nodes_to_terms(children, &mut sub_terms, tag_counter, slot_counter);
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

fn parse_vue_element_node(
    chars: &[char],
    start: usize,
    line: &mut usize,
    col: &mut usize,
) -> Option<(usize, VueNode)> {
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
            if i < chars.len() && (chars[i] == '"' || chars[i] == '\'') {
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
            }
        }
    }

    if is_self_closing || tag_name == "script" || tag_name == "style" {
        return Some((
            i,
            VueNode::Element {
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

        if chars[i] == '<' && i + 1 < chars.len() && chars[i + 1].is_ascii_alphabetic() {
            if let Some((new_idx, child_node)) = parse_vue_element_node(chars, i, line, col) {
                children.push(child_node);
                i = new_idx;
                continue;
            }
        }

        // Vue Mustache interpolation {{ expr }}
        if chars[i] == '{' && i + 1 < chars.len() && chars[i + 1] == '{' {
            i += 2;
            let mut expr = String::new();
            while i + 1 < chars.len() && !(chars[i] == '}' && chars[i + 1] == '}') {
                expr.push(chars[i]);
                i += 1;
            }
            if i + 1 < chars.len() {
                i += 2;
            }
            let trimmed = expr.trim();
            // Handle calls like $t('Billing & Invoices')
            let name = if trimmed.starts_with("$t(") || trimmed.starts_with("t(") {
                let inner = trimmed
                    .trim_start_matches("$t(")
                    .trim_start_matches("t(")
                    .trim_end_matches(')')
                    .trim_matches(|c| c == '\'' || c == '"');
                inner.to_string()
            } else {
                sanitize_name(trimmed, slot_counter)
            };
            children.push(VueNode::Slot(name));
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
            children.push(VueNode::Text(norm));
        }

        if i < chars.len() && chars[i] == '<' {
            if i + 1 < chars.len() && chars[i + 1] == '/' {
                break;
            }
        }
    }

    Some((
        i,
        VueNode::Element {
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
    fn test_extract_vue() {
        let vue = r#"
        <template>
            <div class="profile-card">
                <h1>User Profile</h1>
                <p>Welcome, {{ user.name }}! You have {{ unreadCount }} new messages.</p>
                <button title="Click to view details">View Details</button>
            </div>
        </template>
        <script>
        export default { name: 'ProfileCard' }
        </script>
        "#;

        let items = extract_vue(vue, "ProfileCard.vue");
        assert!(!items.is_empty());
        let texts: Vec<&str> = items.iter().map(|it| it.canonical_text.as_str()).collect();
        assert!(texts.iter().any(|t| t.contains("User Profile")));
        assert!(texts.iter().any(|t| t.contains("Welcome, {user.name}! You have {unreadCount} new messages.")));
        assert!(texts.iter().any(|t| t.contains("View Details")));
        assert!(texts.iter().any(|t| t.contains("Click to view details")));
    }
}
