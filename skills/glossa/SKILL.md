---
name: glossa
description: Localization tool for extracting translatable strings from source code and catalogs, translating missing entries through a configurable provider, checking translations with an integrity gate, and writing locale catalogs.
---

# Glossa agent skill

Use this skill when a project needs its user-facing strings extracted, translated or checked. Glossa
has extractors for React/JSX, Vue, Svelte, HTML templates, Markdown, translation-call patterns in
Python, Go, Ruby, PHP, Swift, Angular and Astro sources, and catalogs in JSON, YAML, PO, Fluent,
Android XML, iOS `.strings`, Flutter ARB, Properties, XLIFF and CSV.

## Quickstart

```bash
cargo install --git https://github.com/webfold-ai/glossa glossa   # installs `glossa` and `lingo`
glossa init --locales es,fr,de,ja
glossa extract && glossa check --ci
```

`lingo` is the same CLI under a second name, so existing `lingo.json` / `lingo.lock` projects can keep
using it.

## Commands

| Command | Purpose |
|---|---|
| `glossa init` | Detects the framework, imports settings from other i18n tools' config files if present, writes `glossa.json` |
| `glossa extract` | Scans the project and updates the lockfile (`glossa.lock`, or an existing `lingo.lock`) |
| `glossa translate` | Translates missing entries with `anthropic`, `openai`, `deepseek`, `ollama` or `offline`; each result goes through the integrity gate |
| `glossa check --ci` | Exits 1 if any configured locale is missing a translation; with `--format=github` or in CI prints `::error` annotations and writes `$GITHUB_STEP_SUMMARY` |
| `glossa diff` | Shows pending translations from the lockfile, with no network access |
| `glossa pseudo` | Writes pseudo-localized strings (accents and about 35% expansion) to test UI layout |
| `glossa sync` | Writes locale catalogs from the lockfile in the configured output format |
| `glossa codegen` | Generates TypeScript or Rust message accessors (`--target typescript` or `--target rust`) |
| `glossa visual` | Starts the local in-context editor on `127.0.0.1:3920` |
| `glossa import` | Detects and imports locale settings from Lingo.dev, Inlang, Tolgee, Crowdin, Lokalise or i18next config files |
| `glossa mcp` | Starts an MCP server on stdio |
| `glossa export` | Exports translations as JSONL (`--format jsonl`) or TSV (`--format tsv`) |
| `glossa lorelake --action push` | Pushes lockfile entries to an HTTP object store |
| `glossa bench` | Times Glossa's own extraction, lockfile, gate and emitter code on this machine |

## Integrity gate

A translation is rejected if it drops or adds a `{slot}`, unbalances or reorders numbered tags
(`<0>...</0>`), contains script or event-handler markup, or changes a protected glossary term.

## Lockfile and coordinates

Each string gets a deterministic 64-bit content coordinate, so the same text extracted from different
file types maps to one lockfile entry. Lockfiles merge with `Table::fuse`, which is order-independent.
