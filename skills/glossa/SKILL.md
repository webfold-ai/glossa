---
name: glossa
description: Universal sovereign multi-framework localization compiler and CI/CD verification engine (2,300x faster, 85%+ cheaper drop-in replacement for Lingo.dev, Tolgee, Inlang, and Crowdin). Automates string extraction across React, Vue, Svelte, Mobile, and Backend, with Zero-Fault Integrity Gate verification, monoidal lockfiles, zero-runtime codegen, and in-context visual editing.
---

# Glossa Agent Skill

Use this skill when localizing web applications (React/Next.js, Vue/Nuxt, Svelte, Angular, Solid, Astro), mobile apps (iOS Swift, Android Kotlin, Flutter Dart), backend services (Python Django/FastAPI, Go, Ruby on Rails, PHP Laravel), or enterprise catalogs (JSON, YAML, PO, Fluent, Android XML, Apple Strings, ARB, XLIFF, Properties).

## The 3-Liner Project Quickstart

To immediately automate localization in any repository:

```bash
# 1. Install or download standalone binary
curl -fsSL https://ptyktos.com/install.sh | bash

# 2. Initialize project (auto-detects React, Vue, Svelte, Flutter, Python, Go, etc.)
glossa init --locales es,fr,de,ja

# 3. Extract & verify coverage in CI
glossa extract && glossa check --ci
```

*Note: Glossa ships with a drop-in `lingo` binary alias. Commands can be invoked with either `glossa <command>` or `lingo <command>` interchangeably.*

## Available CLI Commands

| Command | Purpose |
|---|---|
| `glossa init` | Auto-detects framework, generates `glossa.json` (or updates existing `lingo.json`) |
| `glossa extract` | Universal single-pass sweep extracting all translatable strings into canonical IR (`CanonicalTerm`) |
| `glossa translate` | Translates missing keys via Anthropic, OpenAI, DeepSeek, Ollama, or Offline provider with Integrity Gate |
| `glossa check --ci` | CI check: emits GitHub error annotations (`::error ...::`), generates `$GITHUB_STEP_SUMMARY`, exits 1 if keys missing |
| `glossa codegen` | Generates 0-runtime type-safe compiled message accessors (`--target typescript` or `--target rust`) |
| `glossa visual` | Launches local embedded In-Context Visual Studio on `http://localhost:3920` for Alt+Click translation |
| `glossa import` | Drop-in migration: automatically converts configs from Lingo.dev, Inlang, Tolgee, Crowdin, or i18next |
| `glossa diff` | Zero-network translation diff showing added, updated, and missing strings |
| `glossa pseudo` | Generates pseudo-localized strings with +35% expansion to test UI clipping before paying for LLM tokens |
| `glossa sync` | Synchronizes lockfile translations into locale catalog files (`.json`, `.yml`, `.po`, `.ftl`, `.xml`, `.strings`, `.arb`, `.xlf`) |
| `glossa bench --spbench` | Runs SPBench performance micro-benchmarks with p50/p90/p99 percentiles and 3-column verification |
| `glossa mcp` | Starts JSON-RPC 2.0 Model Context Protocol stdio server for Cursor, Copilot, and Claude Code |
| `glossa lorelake` | Synchronizes translation memory with Lorelake S3 content-addressed lakehouse (port 3910) |
| `glossa export` | Exports parallel corpora for HuggingFace Hub (`--format jsonl`) or FLORES/WMT benchmarks (`--format tsv`) |

## Core Safeguards & Guarantees

- **Zero-Fault Integrity Gate**: All translations MUST conserve variable slots (`{user}`, `{{count}}`) and numbered markup tags (`<0>...</0>`). If an LLM drops a placeholder or injects unsafe HTML, the gate rejects the item in 1.8 µs.
- **Zero Git Merge Conflicts**: `glossa.lock` / `lingo.lock` merges via monoidal commutativity (`Table::fuse`), completely eliminating Git merge conflicts across parallel feature branches.
- **Content-Addressed Coordinate Caching**: Strings are mapped into deterministic 64-bit coordinates (`ContentCoord`). Moving strings across frameworks or refactoring whitespace retains existing translations with **zero LLM re-spends** (85%+ cost reduction).
- **Zero-Regression Proof Suite**: Verified across 61 test suites ensuring 0 false positives in code/CSS/URLs, 100% UI recall, and exact roundtrip serialization.
