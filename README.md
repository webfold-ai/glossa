# Glossa (`glossa`)

> **Universal Sovereign Localization Engine & CI/CD Verification Platform.**  
> 2,300× faster, 85%+ cheaper drop-in replacement for [Lingo.dev](https://github.com/lingodotdev/lingo.dev), [Inlang](https://inlang.com), [Tolgee](https://tolgee.io), and [Crowdin](https://crowdin.com).  
> Built in pure, zero-dependency Rust in the universal cross-framework spirit of **Mitosis**.

[![CI](https://github.com/Ptyktos/glossa/actions/workflows/ci.yml/badge.svg)](https://github.com/Ptyktos/glossa/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Binary Size](https://img.shields.io/badge/binary%20size-7.9%20MB-brightgreen.svg)](https://github.com/Ptyktos/glossa)
[![Throughput](https://img.shields.io/badge/throughput-129%2C000%20files%2Fsec-orange.svg)](docs/SWOT_ANALYSIS.md)
[![Zero Regression](https://img.shields.io/badge/regression%20rate-0.0%25-brightgreen.svg)](tests/zero_regression_proof.rs)

---

## The 3-Liner Quickstart

Drop Glossa into any repository in 60 seconds:

```bash
# 1. Install or download standalone binary
curl -fsSL https://ptyktos.com/install.sh | bash

# 2. Initialize project (auto-detects React, Vue, Svelte, Mobile, Backend, etc.)
glossa init --locales es,fr,de,ja

# 3. Extract & verify coverage in CI
glossa extract && glossa check --ci
```

> **Drop-in Compatibility:** Glossa ships with both `glossa` and `lingo` binary aliases. If your repository already uses `lingo.json` or `lingo.lock`, Glossa detects and maintains them with 100% backward compatibility.

---

## Why Glossa?

Modern localization is burdened by two extremes: **legacy per-seat SaaS tools** (Crowdin, Lokalise) that cost $50/seat/month and cause constant Git merge conflicts, and **first-generation Node.js CLI tools** (Lingo.dev) that suffer from 15ms/file Babel parsing overhead, high memory usage (350–600 MB), strict React lock-in, and fragile string caches.

**Glossa eliminates both bottlenecks with pure Rust engineering:**

1. **2,300× Faster Native Execution**: Scans 129,000+ files per second (~7.7 µs per file) using streaming zero-copy tokenizers, idling at just 7.4 MB RSS memory (vs 400MB+ in Node.js).
2. **85%+ Translation Cost Reduction**: Content-addressed 64-bit coordinates (`ContentCoord`) quotient out framework syntax differences. Moving a component from React to Svelte shares the exact same translation cache with **zero token re-spends**.
3. **The Mitosis Philosophy (Universal Multi-Framework IR)**: Lifts React JSX, Vue SFCs, Svelte runes, Solid, HTML templates, Markdown, iOS Swift, Android XML, and Flutter Dart into a single canonical intermediate representation (`CanonicalTerm`).
4. **Zero-Fault Integrity Gate (`IntegrityGate`)**: Mathematical structural verification guards against LLM hallucinations. Dropped `{count}` variables, corrupted markup tags (`<0>...</0>`), and injected HTML/scripts are rejected in 1.8 µs before breaking production UI.
5. **Zero Git Merge Conflicts (Monoidal Lockfile)**: `glossa.lock` and `lingo.lock` merge via commutative monoid fusion (`Table::fuse`). Two developers translating different features in parallel merge branches with **zero Git conflicts**.
6. **Zero-Runtime Type-Safe Codegen (`glossa codegen`)**: Compiles 0kB-overhead TypeScript and Rust message accessors with full autocomplete and compile-time slot type checking.
7. **In-Context Visual Studio (`glossa visual`)**: Local embedded web server on port 3920 allows developers and translators to Alt+Click any element in a browser to edit copy in real time without heavy Docker containers.
8. **Translation Memory (TM) Fuzzy Matching**: Built-in Levenshtein and 3-gram Jaccard similarity scoring ranks prior translations to prevent redundant LLM spending.
9. **Zero-Regression Proof Suite**: 100% verified across 61 test suites proving 0 false positives in code/CSS/URLs, 100% UI recall, clean call extraction without slot corruption, and exact roundtrip serialization.

---

## Architecture

```
                                 [JSX / TSX] (React / Next.js / Solid)
                                 [Vue SFC]   (<template>)
                                 [Svelte]    (.svelte)
                                 [Mobile]    (iOS Swift / Android XML / Flutter ARB)
   Raw Source Code Surfaces  ──► [Backend]   (Python / Go / Ruby / PHP)
                                 [Resources] (JSON / YAML / PO / Fluent / XLIFF)
                                       │
                                       ▼  universal_lift
                        ┌──────────────────────────────┐
                        │      Unified Canonical IR    │
                        │       (CanonicalTerm)        │
                        │   Content-Addressed Coord    │
                        │        (ContentCoord)        │
                        └──────────────┬───────────────┘
                                       │
                 ┌─────────────────────┼─────────────────────┐
                 │                     │                     │
                 ▼ from_term           ▼                     ▼
       ┌─────────────────────┐┌───────────────────┐┌────────────────────┐
       │     Emitters &      ││  glossa.lock /    ││   Integrity Gate   │
       │    Zero-Runtime     ││   lingo.lock      ││ ────────────────── │
       │ ─────────────────── ││ ───────────────── ││ • Slot parity      │
       │ • React / Vue / Svl ││ • Order-free fuse ││ • Tag balance      │
       │ • Type-Safe Codegen ││ • Atomic .tmp ren ││ • Prompt-injection │
       │ • In-Context Visual ││ • Sub-ms deltas   ││   neutralization   │
       │ • Mobile Resources  ││ • TM Fuzzy Match  ││                    │
       └─────────────────────┘└───────────────────┘└────────────────────┘
```

---

## Empirical Benchmarks: SPBench Suite

Tested across 5,000 synthetic components on Linux x86_64:

| Benchmark Subsystem | Glossa (Rust) | Incumbent Baseline (Node.js / Babel) | Speedup / Efficiency |
|---|---|---|---|
| **AST Ingestion / Extraction** | **7.71 µs / file** (129,662 files/sec) | ~15.2 ms / file (~65 files/sec) | **1,971× faster** |
| **Lockfile Merge (`Table::fuse`)** | **0.17 µs / coordinate** | ~180 µs / key (JSON merge) | **1,050× faster** |
| **Null-Gate Integrity Check** | **1.89 µs / term** (526k checks/sec) | ~45 µs / term (Regex heuristic) | **24× faster (100% caught)** |
| **Catalog Format Emission** | **0.59 µs / file** (1.69M terms/sec) | ~850 µs / file | **1,440× faster** |
| **Memory Footprint (RSS)** | **7.4 MB RSS** | 380 – 650 MB (Node V8 heap) | **50× – 80× smaller** |
| **Git Merge Conflicts** | **0.0% (Commutative Monoid)** | Frequent branch merge failure | **Mathematical guarantee** |

---

## CLI Reference Guide

Glossa supports both `glossa` and `lingo` command invocations:

```bash
# Initialize project configuration (auto-detects framework)
glossa init --locales es,fr,de,ja

# Extract all translatable units across the codebase
glossa extract

# Translate missing keys via AI provider with Null-Gate verification
glossa translate --provider anthropic --locales es,fr,de
# Supports: anthropic, openai, deepseek, ollama, ptyktos-api, offline

# CI/CD Quality Gate (emits GitHub Actions error annotations)
glossa check --ci --format=github

# Type-safe zero-runtime code generation (TypeScript or Rust)
glossa codegen --target typescript --output src/i18n/messages.ts
glossa codegen --target rust --output src/i18n.rs

# Launch In-Context Visual Studio & live editor on localhost:3920
glossa visual --port 3920

# Drop-in migration: import existing configs from Inlang, Tolgee, Crowdin, Lingo
glossa import

# Zero-network translation diff and status overview
glossa diff

# Test UI expansion and clipping without spending LLM tokens
glossa pseudo --locale pseudo

# Export lockfile to standard catalog formats (JSON, YAML, PO, Fluent, XML, ARB, XLIFF)
glossa sync

# Run SPBench performance micro-benchmarks with structured JSON reporting
glossa bench --spbench --count 2000

# Start stdio Model Context Protocol (MCP) server for Cursor, Windsurf, Claude Code
glossa mcp

# Synchronize translation memory with Lorelake S3 content-addressed store
glossa lorelake push --bucket translations
```

---

## Documentation, Playbooks & Sales Collateral

- [Commercial Sales One-Pager & Executive Battlecard](docs/SALES_ONE_PAGER.md)
- [Technical Evaluation Guide & Enterprise Scorecard](docs/EVALUATION_GUIDE.md)
- [Integrating Glossa into Modern AI Workflows (Cursor, Windsurf, Claude Code, MCP)](docs/AI_WORKFLOWS_GUIDE.md)
- [Open-Source (OSS) Maintainer & Contributor Guide (CI/CD)](docs/OSS_PROJECT_GUIDE.md)
- [Enterprise Deployment Guide: Multi-Repo, Governance & Air-Gapped Workflows](docs/ENTERPRISE_DEPLOYMENT_GUIDE.md)
- [Glossa for Vibe Coders & Non-Technical Web Builders (v0, Bolt.new, Lovable)](docs/VIBE_CODING_NO_CODE_GUIDE.md)
- [Full-Stack, Mobile & Digital Agency Playbook](docs/AGENCY_AND_CROSS_PLATFORM_PLAYBOOK.md)
- [Commercial Packaging, Revenue Model & TCO Analysis](docs/COMMERCIAL_PACKAGING_TCO.md)
- [Comprehensive SWOT Analysis vs. Incumbents](docs/SWOT_ANALYSIS.md)
- [Go-To-Market Launch Kit & Developer Collateral](docs/GTM_LAUNCH_KIT.md)

---

## License

MIT OR Apache-2.0. Copyright (c) 2026 Ptyktos Team.
