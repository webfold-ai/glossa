//! High-Speed Markdown / MDX Parser & Extractor.
//!
//! Extracts translatable paragraphs, headings, list items, and blockquotes,
//! preserving YAML frontmatter, code blocks, and markdown link structures.

use crate::ir::coord::ApeironCoord;
use crate::ir::term::{normalize_whitespace, ApeironTerm, ExtractedItem, SourceLocation};

pub fn extract_markdown(source: &str, file_path: &str) -> Vec<ExtractedItem> {
    let mut items = Vec::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    // Check for YAML frontmatter `---`
    if !lines.is_empty() && lines[0].trim() == "---" {
        i += 1;
        while i < lines.len() && lines[i].trim() != "---" {
            i += 1;
        }
        if i < lines.len() {
            i += 1; // skip closing '---'
        }
    }

    let mut in_code_block = false;
    let mut current_block = String::new();
    let mut block_start_line = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();
        let line_num = i + 1;

        // Fenced code blocks
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            if !current_block.is_empty() {
                flush_block(&mut items, &current_block, file_path, block_start_line);
                current_block.clear();
            }
            in_code_block = !in_code_block;
            i += 1;
            continue;
        }

        if in_code_block {
            i += 1;
            continue;
        }

        // Empty line separates blocks
        if trimmed.is_empty() {
            if !current_block.is_empty() {
                flush_block(&mut items, &current_block, file_path, block_start_line);
                current_block.clear();
            }
            i += 1;
            continue;
        }

        // Heading lines `# ...`
        if trimmed.starts_with('#') {
            if !current_block.is_empty() {
                flush_block(&mut items, &current_block, file_path, block_start_line);
                current_block.clear();
            }
            let text = trimmed.trim_start_matches('#').trim();
            flush_block(&mut items, text, file_path, line_num);
            i += 1;
            continue;
        }

        // Accumulate paragraph
        if current_block.is_empty() {
            block_start_line = line_num;
            current_block.push_str(trimmed);
        } else {
            current_block.push(' ');
            current_block.push_str(trimmed);
        }

        i += 1;
    }

    if !current_block.is_empty() {
        flush_block(&mut items, &current_block, file_path, block_start_line);
    }

    items
}

fn flush_block(items: &mut Vec<ExtractedItem>, text: &str, file_path: &str, line_num: usize) {
    let norm = normalize_whitespace(text);
    if norm.trim().is_empty() {
        return;
    }

    // Parse links and inline formatting into ApeironTerm nodes
    let terms = parse_markdown_inlines(&norm);
    let canon = ApeironTerm::to_canonical_string(&terms);
    let coord = ApeironCoord::from_str_canonical(&canon);

    items.push(ExtractedItem {
        coord,
        key: None,
        terms,
        canonical_text: canon,
        context: Some(format!("Markdown in {file_path}:{line_num}")),
        locations: vec![SourceLocation {
            file_path: file_path.to_string(),
            line: line_num,
            column: 1,
        }],
    });
}

fn parse_markdown_inlines(s: &str) -> Vec<ApeironTerm> {
    let mut terms = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut text_buf = String::new();
    let mut tag_counter = 0;

    while i < chars.len() {
        // Check for link `[text](url)`
        if chars[i] == '[' {
            let mut j = i + 1;
            let mut link_text = String::new();
            while j < chars.len() && chars[j] != ']' {
                link_text.push(chars[j]);
                j += 1;
            }
            if j + 1 < chars.len() && chars[j] == ']' && chars[j + 1] == '(' {
                let mut k = j + 2;
                let mut url = String::new();
                while k < chars.len() && chars[k] != ')' {
                    url.push(chars[k]);
                    k += 1;
                }
                if k < chars.len() && chars[k] == ')' {
                    if !text_buf.is_empty() {
                        terms.push(ApeironTerm::Text(text_buf.clone()));
                        text_buf.clear();
                    }
                    let tag_id = tag_counter;
                    tag_counter += 1;
                    terms.push(ApeironTerm::Tag {
                        id: tag_id,
                        tag: "a".to_string(),
                        attrs: vec![("href".to_string(), url)],
                        children: vec![ApeironTerm::Text(link_text)],
                    });
                    i = k + 1;
                    continue;
                }
            }
        }

        // Inline code `code`
        if chars[i] == '`' {
            let mut j = i + 1;
            let mut code = String::new();
            while j < chars.len() && chars[j] != '`' {
                code.push(chars[j]);
                j += 1;
            }
            if j < chars.len() && chars[j] == '`' {
                if !text_buf.is_empty() {
                    terms.push(ApeironTerm::Text(text_buf.clone()));
                    text_buf.clear();
                }
                let tag_id = tag_counter;
                tag_counter += 1;
                terms.push(ApeironTerm::Tag {
                    id: tag_id,
                    tag: "code".to_string(),
                    attrs: Vec::new(),
                    children: vec![ApeironTerm::Text(code)],
                });
                i = j + 1;
                continue;
            }
        }

        text_buf.push(chars[i]);
        i += 1;
    }

    if !text_buf.is_empty() {
        terms.push(ApeironTerm::Text(text_buf));
    }

    terms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_markdown() {
        let md = r#"---
title: Documentation
author: Ptyktos
---

# Getting Started

Welcome to the **fastest** localization platform on earth. Check the [documentation](https://docs.ptyktos.com) for more details.

```bash
cargo run --release
```

This is the final paragraph.
"#;

        let items = extract_markdown(md, "README.md");
        assert!(!items.is_empty());
        let texts: Vec<&str> = items.iter().map(|it| it.canonical_text.as_str()).collect();
        assert!(texts.iter().any(|t| t == &"Getting Started"));
        assert!(texts
            .iter()
            .any(|t| t.contains("Check the <0>documentation</0> for more details.")));
        assert!(texts
            .iter()
            .any(|t| t.contains("This is the final paragraph.")));
    }
}
