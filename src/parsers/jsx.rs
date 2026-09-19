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

/// Real JSX tags are never glued directly onto a preceding identifier
/// character — they're preceded by whitespace, `(`, `{`, `>`, `&&`, `?`,
/// `:`, a newline, or start-of-file. A `<` immediately after an identifier
/// character is TypeScript generic syntax (`useState<string>`, `Array<T>`,
/// `Map<K, V>` — ubiquitous in real .tsx/.ts files) or a bare comparison
/// (`i<len`, `x<y`), never a real tag. Found by testing against real
/// production files in gitroomhq/postiz-app: dozens of real .tsx component
/// files each individually timed out because every generic type parameter
/// in the file was optimistically parsed as a tag-open, then scanned to EOF
/// hunting for a closing tag that could never exist — O(file size) wasted
/// work per generic usage, compounding across every generic in the file.
fn preceded_by_identifier_char(chars: &[char], lt_idx: usize) -> bool {
    lt_idx > 0 && (chars[lt_idx - 1].is_alphanumeric() || chars[lt_idx - 1] == '_' || chars[lt_idx - 1] == '$')
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

        if chars[idx] == '<' && idx + 1 < len && !preceded_by_identifier_char(&chars, idx) {
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
        if chars[i] == '<'
            && i + 1 < chars.len()
            && chars[i + 1].is_ascii_alphabetic()
            && !preceded_by_identifier_char(chars, i)
        {
            if let Some((new_idx, child_node)) = parse_jsx_element_node(chars, i, line, col) {
                children.push(child_node);
                i = new_idx;
                continue;
            }
            // A REAL, always-reproducible infinite loop, found via CI
            // bisection against gitroomhq/postiz-app: parse_jsx_element_node
            // can legitimately return None here (e.g. a self-closing tag
            // whose only attribute is a bare spread `{...x}` — no name, no
            // `=`, correctly rejected as not-real-JSX-attribute-syntax by
            // the fix in the attribute loop above). When it does, `i` is
            // left pointing at the unconsumed `<`. The "Plain text
            // character" fallback below stops its own loop the instant it
            // sees a `<` (by design, so real child elements get handled by
            // the branch above) — so with `i` unmoved and every branch
            // re-evaluating identically, the outer `while i < chars.len()`
            // loop above spins forever at the same position. Every real
            // .tsx file with a self-closing tag using ONLY a spread
            // attribute, followed by any sibling, hits this. Consume the
            // `<` as a single literal text character so the loop can
            // actually make progress.
            children.push(JsxNode::Text("<".to_string()));
            i += 1;
            *col += 1;
            continue;
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
            // skip_curly returning None means an unbalanced `{` (no matching
            // `}` before EOF) -- same shape of bug: without an explicit
            // advance here, the outer loop would spin on this `{` forever
            // too, for the same reason as the `<` case above.
            children.push(JsxNode::Text("{".to_string()));
            i += 1;
            *col += 1;
            continue;
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

    /// Regression guard for the second, larger real-world hang: TypeScript
    /// generic type parameters (`useState<string>`, `Array<T>`,
    /// `Map<K, V>`) are ubiquitous in real .tsx/.ts files. Each one was
    /// optimistically parsed as a self-closing-ish tag-open, then scanned to
    /// EOF hunting for a `</string>`/`</T>`/`</K>` closing tag that could
    /// never exist -- not an infinite loop like the comparison-operator bug,
    /// but O(file size) wasted work per generic usage. Found via CI
    /// bisection against gitroomhq/postiz-app: dozens of real .tsx
    /// component files each individually timed out at 3 seconds.
    #[test]
    fn test_regression_typescript_generics_are_not_treated_as_jsx_tags() {
        let source = r#"
        import { useState, useRef } from 'react';
        function Component() {
            const [name, setName] = useState<string>('');
            const items = useRef<Array<number>>([]);
            const cache: Map<string, Record<string, boolean>> = new Map();
            return <div>{name}</div>;
        }
        "#;
        let items = extract_jsx(source, "Component.tsx");
        // The only real JSX here is the final `<div>{name}</div>` -- the
        // generics above it must not be mistaken for tags, and (this is the
        // actual regression) parsing must complete at all rather than
        // scanning to EOF for each one.
        assert!(items.iter().all(|i| i.canonical_text != "string" && i.canonical_text != "number"));
    }

    /// Diagnostic regression test: verbatim content of
    /// `gitroomhq/postiz-app`'s `libraries/react-shared-libraries/src/form/textarea.tsx`,
    /// which still hits a 20-second timeout when run through the real CLI
    /// (`glossa extract`) in a completely isolated GitHub Actions run — no
    /// CI contention possible, so this is a genuine remaining hang, not
    /// noise. Runs extract_jsx on a background thread with a 5-second
    /// bound: if the JSX parser itself is the culprit, this test fails fast
    /// with a clear message instead of hanging cargo test indefinitely. If
    /// this test PASSES quickly, the bug is elsewhere in the CLI path
    /// (config loading, lockfile I/O, directory walking) — not the parser.
    #[test]
    fn test_regression_textarea_real_world_file_does_not_hang() {
        let source = r#"
'use client';

import { DetailedHTMLProps, FC, InputHTMLAttributes, useMemo } from 'react';
import clsx from 'clsx';
import { useFormContext } from 'react-hook-form';
import { TranslatedLabel } from '../translation/translated-label';

export const Textarea: FC<
  DetailedHTMLProps<
    InputHTMLAttributes<HTMLTextAreaElement>,
    HTMLTextAreaElement
  > & {
    error?: any;
    disableForm?: boolean;
    label: string;
    name: string;
    translationKey?: string;
    translationParams?: Record<string, string | number>;
  }
> = (props) => {
  const {
    label,
    className,
    disableForm,
    error,
    translationKey,
    translationParams,
    ...rest
  } = props;
  const form = useFormContext();
  const err = useMemo(() => {
    if (error) return error;
    if (!form || !form.formState.errors[props?.name!]) return;
    return form?.formState?.errors?.[props?.name!]?.message! as string;
  }, [form?.formState?.errors?.[props?.name!]?.message, error]);
  return (
    <div
      className={clsx(
        'flex flex-col gap-[6px]',
        props.disabled && 'opacity-50'
      )}
    >
      <div className={`text-[14px]`}>
        <TranslatedLabel
          label={label}
          translationKey={translationKey}
          translationParams={translationParams}
        />
      </div>
      <textarea
        {...(disableForm ? {} : form.register(props.name))}
        className={clsx(
          'bg-input min-h-[150px] p-[16px] outline-none border-fifth border rounded-[4px] text-inputText placeholder-inputText',
          className
        )}
        {...rest}
      />
      <div className="text-red-400 text-[12px]">{err || <>&nbsp;</>}</div>
    </div>
  );
};
"#;
        let (tx, rx) = std::sync::mpsc::channel();
        let src = source.to_string();
        std::thread::spawn(move || {
            let items = extract_jsx(&src, "textarea.tsx");
            let _ = tx.send(items.len());
        });
        match rx.recv_timeout(std::time::Duration::from_secs(5)) {
            Ok(_) => {}
            Err(_) => panic!("extract_jsx hung for >5s on textarea.tsx — the bug is in the JSX parser itself"),
        }
    }

    /// Shared helper: run extract_jsx on a background thread with a 3s
    /// bound, panicking with a clear message (not hanging) if it doesn't
    /// return in time.
    fn extract_jsx_bounded(source: &'static str, label: &'static str) {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let items = extract_jsx(source, "test.tsx");
            let _ = tx.send(items.len());
        });
        if rx.recv_timeout(std::time::Duration::from_secs(3)).is_err() {
            panic!("extract_jsx hung for >3s on case: {label}");
        }
    }

    #[test]
    fn test_bisect_case_1_type_signature_only() {
        extract_jsx_bounded(
            r#"
            export const Textarea: FC<
              DetailedHTMLProps<
                InputHTMLAttributes<HTMLTextAreaElement>,
                HTMLTextAreaElement
              > & {
                error?: any;
                translationParams?: Record<string, string | number>;
              }
            > = (props) => {
              return <div></div>;
            };
            "#,
            "type_signature_only",
        );
    }

    #[test]
    fn test_bisect_case_2_textarea_spread_tag_only() {
        extract_jsx_bounded(
            r#"
            function C(props: any) {
              const { className, ...rest } = props;
              return (
                <textarea
                  {...(props.disableForm ? {} : form.register(props.name))}
                  className={clsx('a', className)}
                  {...rest}
                />
              );
            }
            "#,
            "textarea_spread_tag_only",
        );
    }

    #[test]
    fn test_bisect_case_3_fragment_in_expression_only() {
        extract_jsx_bounded(
            r#"
            function C() {
              return <div className="a">{err || <>&nbsp;</>}</div>;
            }
            "#,
            "fragment_in_expression_only",
        );
    }

    #[test]
    fn test_bisect_case_4_nested_divs_and_component_only() {
        extract_jsx_bounded(
            r#"
            function C(props: any) {
              return (
                <div className={clsx('flex flex-col gap-[6px]', props.disabled && 'opacity-50')}>
                  <div className={`text-[14px]`}>
                    <TranslatedLabel label={label} translationKey={translationKey} translationParams={translationParams} />
                  </div>
                </div>
              );
            }
            "#,
            "nested_divs_and_component_only",
        );
    }

    #[test]
    fn test_bisect_case_5_optional_chaining_index_only() {
        extract_jsx_bounded(
            r#"
            function C(props: any) {
              const err = useMemo(() => {
                if (!form || !form.formState.errors[props?.name!]) return;
                return form?.formState?.errors?.[props?.name!]?.message! as string;
              }, [form?.formState?.errors?.[props?.name!]?.message, error]);
              return <div>{err}</div>;
            }
            "#,
            "optional_chaining_index_only",
        );
    }

    /// All 5 isolated bisection cases above pass individually, yet the full
    /// real file still hangs -- so it's an interaction, not a single
    /// construct. Testing by REMOVING one real section at a time from the
    /// verbatim file instead of building up from fragments, to preserve
    /// realistic surrounding state.
    #[test]
    fn test_bisect_case_6_full_file_minus_complex_type_signature() {
        extract_jsx_bounded(
            r#"
export const Textarea: any = (props: any) => {
  const {
    label,
    className,
    disableForm,
    error,
    translationKey,
    translationParams,
    ...rest
  } = props;
  const form = useFormContext();
  const err = useMemo(() => {
    if (error) return error;
    if (!form || !form.formState.errors[props?.name!]) return;
    return form?.formState?.errors?.[props?.name!]?.message! as string;
  }, [form?.formState?.errors?.[props?.name!]?.message, error]);
  return (
    <div
      className={clsx(
        'flex flex-col gap-[6px]',
        props.disabled && 'opacity-50'
      )}
    >
      <div className={`text-[14px]`}>
        <TranslatedLabel
          label={label}
          translationKey={translationKey}
          translationParams={translationParams}
        />
      </div>
      <textarea
        {...(disableForm ? {} : form.register(props.name))}
        className={clsx(
          'bg-input min-h-[150px] p-[16px] outline-none border-fifth border rounded-[4px] text-inputText placeholder-inputText',
          className
        )}
        {...rest}
      />
      <div className="text-red-400 text-[12px]">{err || <>&nbsp;</>}</div>
    </div>
  );
};
"#,
            "full_file_minus_complex_type_signature",
        );
    }

    #[test]
    fn test_bisect_case_7_full_type_signature_plus_trivial_body() {
        extract_jsx_bounded(
            r#"
export const Textarea: FC<
  DetailedHTMLProps<
    InputHTMLAttributes<HTMLTextAreaElement>,
    HTMLTextAreaElement
  > & {
    error?: any;
    disableForm?: boolean;
    label: string;
    name: string;
    translationKey?: string;
    translationParams?: Record<string, string | number>;
  }
> = (props) => {
  const {
    label,
    className,
    disableForm,
    error,
    translationKey,
    translationParams,
    ...rest
  } = props;
  const form = useFormContext();
  const err = useMemo(() => {
    if (error) return error;
    if (!form || !form.formState.errors[props?.name!]) return;
    return form?.formState?.errors?.[props?.name!]?.message! as string;
  }, [form?.formState?.errors?.[props?.name!]?.message, error]);
  return <div>{err}</div>;
};
"#,
            "full_type_signature_plus_trivial_body",
        );
    }

    /// Case 7 (real type sig + trivial body) passed; case 6 (trivial type
    /// sig + real body) hangs -- so the bug is entirely within the body:
    /// destructuring + useMemo + the full real JSX, together. Narrowing
    /// further within the body.
    #[test]
    fn test_bisect_case_8_body_without_destructuring() {
        extract_jsx_bounded(
            r#"
export const Textarea: any = (props: any) => {
  const form = useFormContext();
  const err = useMemo(() => {
    if (props.error) return props.error;
    if (!form || !form.formState.errors[props?.name!]) return;
    return form?.formState?.errors?.[props?.name!]?.message! as string;
  }, [form?.formState?.errors?.[props?.name!]?.message, props.error]);
  return (
    <div
      className={clsx(
        'flex flex-col gap-[6px]',
        props.disabled && 'opacity-50'
      )}
    >
      <div className={`text-[14px]`}>
        <TranslatedLabel
          label={props.label}
          translationKey={props.translationKey}
          translationParams={props.translationParams}
        />
      </div>
      <textarea
        {...(props.disableForm ? {} : form.register(props.name))}
        className={clsx(
          'bg-input min-h-[150px] p-[16px] outline-none border-fifth border rounded-[4px] text-inputText placeholder-inputText',
          props.className
        )}
        {...props}
      />
      <div className="text-red-400 text-[12px]">{err || <>&nbsp;</>}</div>
    </div>
  );
};
"#,
            "body_without_destructuring",
        );
    }

    #[test]
    fn test_bisect_case_9_body_without_usememo() {
        extract_jsx_bounded(
            r#"
export const Textarea: any = (props: any) => {
  const {
    label,
    className,
    disableForm,
    error,
    translationKey,
    translationParams,
    ...rest
  } = props;
  const form = useFormContext();
  const err = null;
  return (
    <div
      className={clsx(
        'flex flex-col gap-[6px]',
        props.disabled && 'opacity-50'
      )}
    >
      <div className={`text-[14px]`}>
        <TranslatedLabel
          label={label}
          translationKey={translationKey}
          translationParams={translationParams}
        />
      </div>
      <textarea
        {...(disableForm ? {} : form.register(props.name))}
        className={clsx(
          'bg-input min-h-[150px] p-[16px] outline-none border-fifth border rounded-[4px] text-inputText placeholder-inputText',
          className
        )}
        {...rest}
      />
      <div className="text-red-400 text-[12px]">{err || <>&nbsp;</>}</div>
    </div>
  );
};
"#,
            "body_without_usememo",
        );
    }

    /// Neither destructuring nor useMemo alone matters -- both removals
    /// still hang (cases 8, 9), and each of the 3 JSX children passed when
    /// tested alone (cases 2, 3, 4). So it must be the SEQUENCE of multiple
    /// sibling children under one parent <div> -- testing pairs to narrow
    /// which combination triggers it.
    #[test]
    fn test_bisect_case_10_label_div_plus_textarea_siblings() {
        extract_jsx_bounded(
            r#"
function C(props: any) {
  return (
    <div className={clsx('flex flex-col gap-[6px]', props.disabled && 'opacity-50')}>
      <div className={`text-[14px]`}>
        <TranslatedLabel label={props.label} translationKey={props.translationKey} translationParams={props.translationParams} />
      </div>
      <textarea
        {...(props.disableForm ? {} : form.register(props.name))}
        className={clsx('bg-input min-h-[150px] p-[16px] outline-none border-fifth border rounded-[4px] text-inputText placeholder-inputText', props.className)}
        {...props}
      />
    </div>
  );
}
"#,
            "label_div_plus_textarea_siblings",
        );
    }

    #[test]
    fn test_bisect_case_11_textarea_plus_fragment_div_siblings() {
        extract_jsx_bounded(
            r#"
function C(props: any) {
  return (
    <div className={clsx('flex flex-col gap-[6px]', props.disabled && 'opacity-50')}>
      <textarea
        {...(props.disableForm ? {} : form.register(props.name))}
        className={clsx('bg-input min-h-[150px] p-[16px] outline-none border-fifth border rounded-[4px] text-inputText placeholder-inputText', props.className)}
        {...props}
      />
      <div className="text-red-400 text-[12px]">{err || <>&nbsp;</>}</div>
    </div>
  );
}
"#,
            "textarea_plus_fragment_div_siblings",
        );
    }

    #[test]
    fn test_bisect_case_12_label_div_plus_fragment_div_siblings() {
        extract_jsx_bounded(
            r#"
function C(props: any) {
  return (
    <div className={clsx('flex flex-col gap-[6px]', props.disabled && 'opacity-50')}>
      <div className={`text-[14px]`}>
        <TranslatedLabel label={props.label} translationKey={props.translationKey} translationParams={props.translationParams} />
      </div>
      <div className="text-red-400 text-[12px]">{err || <>&nbsp;</>}</div>
    </div>
  );
}
"#,
            "label_div_plus_fragment_div_siblings",
        );
    }

    /// Both pairs involving <textarea> hang; the pair without it (case 12)
    /// passed. Isolating to the absolute minimum: a self-closing tag whose
    /// ONLY attribute is a bare spread `{...x}` (no name, no `=`),
    /// immediately followed by a sibling.
    #[test]
    fn test_bisect_case_13_minimal_spread_only_self_closing_plus_sibling() {
        extract_jsx_bounded(
            r#"function C() { return <div><textarea {...props} /><span>x</span></div>; }"#,
            "minimal_spread_only_self_closing_plus_sibling",
        );
    }

    /// Same minimal shape, but the self-closing tag has a real named
    /// attribute before the spread (matching the real file's actual
    /// ordering: className={...} appears before the final {...rest}).
    #[test]
    fn test_bisect_case_14_named_attr_then_spread_self_closing_plus_sibling() {
        extract_jsx_bounded(
            r#"function C() { return <div><textarea className="a" {...props} /><span>x</span></div>; }"#,
            "named_attr_then_spread_self_closing_plus_sibling",
        );
    }

    /// Two files still hit the full 20s timeout after the children-loop
    /// fix (down from ~30): gitroomhq/postiz-app's add.edit.modal.tsx and
    /// [p]/page.tsx. add.edit.modal.tsx has template literals containing
    /// `<p>${line}</p>` inside a .map() callback -- a string, not real JSX
    /// -- which the naive scanner has no way to distinguish from a real
    /// tag. Testing the full file and an isolated slice of just that
    /// pattern to see which specifically hangs.
    #[test]
    fn test_bisect_case_15_full_add_edit_modal_file() {
        extract_jsx_bounded(
            r#"
export const AddEditModalInnerInner = (props: any) => {
  useEffect(() => {
    if (existingData.integration) {
      addInternalValue(
        0,
        existingData.integration,
        existingData.posts.map((post) => ({
          delay: post.delay,
          content:
            post.content.indexOf('<p>') > -1
              ? post.content
              : post.content
                  .split('\n')
                  .map((line: string) => `<p>${line}</p>`)
                  .join(''),
          id: post.id,
          media: post.image as any[],
        }))
      );
    }
    addGlobalValue(
      0,
      props.onlyValues?.length
        ? props.onlyValues.map((p) => ({
            content:
              p.content.indexOf('<p>') > -1
                ? p.content
                : p.content
                    .split('\n')
                    .map((line: string) => `<p>${line}</p>`)
                    .join(''),
            id: makeId(10),
            media: p.image || [],
          }))
        : []
    );
  }, []);
  return (
    <>
      <style>
        {`#support-discord {display: none !important;}`}
      </style>
      <ManageModal {...props} />
    </>
  );
};
"#,
            "full_add_edit_modal_file",
        );
    }

    #[test]
    fn test_bisect_case_16_template_literal_with_jsx_like_tags_only() {
        extract_jsx_bounded(
            r#"
function f() {
  return post.content
    .split('\n')
    .map((line: string) => `<p>${line}</p>`)
    .join('');
}
"#,
            "template_literal_with_jsx_like_tags_only",
        );
    }

    #[test]
    fn test_bisect_case_17_fragment_style_template_then_spread_sibling() {
        extract_jsx_bounded(
            r#"
function f() {
  return (
    <>
      <style>
        {`#support-discord {display: none !important;}`}
      </style>
      <ManageModal {...props} />
    </>
  );
}
"#,
            "fragment_style_template_then_spread_sibling",
        );
    }

    #[test]
    fn test_bisect_case_18_style_template_only_no_fragment() {
        extract_jsx_bounded(
            r#"
function f() {
  return (
    <style>
      {`#support-discord {display: none !important;}`}
    </style>
  );
}
"#,
            "style_template_only_no_fragment",
        );
    }

    #[test]
    fn test_bisect_case_19_fragment_with_two_plain_siblings() {
        extract_jsx_bounded(
            r#"
function f() {
  return (
    <>
      <div>a</div>
      <span>b</span>
    </>
  );
}
"#,
            "fragment_with_two_plain_siblings",
        );
    }

    #[test]
    fn test_bisect_case_20_full_use_effect_body_trivial_return() {
        extract_jsx_bounded(
            r#"
export const AddEditModalInnerInner = (props: any) => {
  useEffect(() => {
    if (existingData.integration) {
      addInternalValue(
        0,
        existingData.integration,
        existingData.posts.map((post) => ({
          delay: post.delay,
          content:
            post.content.indexOf('<p>') > -1
              ? post.content
              : post.content
                  .split('\n')
                  .map((line: string) => `<p>${line}</p>`)
                  .join(''),
          id: post.id,
          media: post.image as any[],
        }))
      );
    }
    addGlobalValue(
      0,
      props.onlyValues?.length
        ? props.onlyValues.map((p) => ({
            content:
              p.content.indexOf('<p>') > -1
                ? p.content
                : p.content
                    .split('\n')
                    .map((line: string) => `<p>${line}</p>`)
                    .join(''),
            id: makeId(10),
            media: p.image || [],
          }))
        : []
    );
  }, []);
  return null;
};
"#,
            "full_use_effect_body_trivial_return",
        );
    }

    #[test]
    fn test_bisect_case_21_use_effect_body_then_fragment_return() {
        extract_jsx_bounded(
            r#"
export const AddEditModalInnerInner = (props: any) => {
  useEffect(() => {
    addGlobalValue(
      0,
      props.onlyValues?.length
        ? props.onlyValues.map((p) => ({
            content:
              p.content.indexOf('<p>') > -1
                ? p.content
                : p.content
                    .split('\n')
                    .map((line: string) => `<p>${line}</p>`)
                    .join(''),
            id: makeId(10),
            media: p.image || [],
          }))
        : []
    );
  }, []);
  return (
    <>
      <style>
        {`#support-discord {display: none !important;}`}
      </style>
      <ManageModal {...props} />
    </>
  );
};
"#,
            "use_effect_body_then_fragment_return",
        );
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
