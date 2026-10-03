# Glossa (`glossa`)

> **Localization extraction and integrity checks for codebases.**
> Glossa scans supported source and catalog files and checks translations for missing entries, placeholder changes, and broken markup.
>
> **Distribution status: not yet public.** The crate depends on a private GitLab repository and cannot be built outside the organization.

[![License: FSL-1.1-ALv2](https://img.shields.io/badge/License-FSL--1.1--ALv2-blue.svg)](LICENSE)

---

## Installation status

**Not yet public.** There is no public installer or downloadable binary. `Cargo.toml` depends on `hashtrinity` from the private, VPN-only GitLab repository `https://gitlab.tas.twn.network/twn/ptyktos/hashtrinity.git`; users outside the organization cannot resolve this dependency or build the crate. Do not use the former `ptyktos.com/install.sh` command.

> The source includes `glossa` and `lingo` binary aliases and supports reading `lingo.json` / `lingo.lock`; this has not been validated against a real-repository corpus.

---

## Why Glossa?

Glossa is a prototype for extracting localization strings and checking translation integrity in CI. No competitor product has been measured against it, and no real-repository corpus has been evaluated.

**Glossa eliminates both bottlenecks with pure Rust engineering:**

1. **Fixture extraction benchmark**: A synthetic in-memory fixture micro-benchmark compared Glossa with `@babel/parser`; it does not measure files, repository scanning, or a competing localization product. See the benchmark scope below.
2. **Content-Addressed Translation Cache**: 64-bit coordinates (`ContentCoord`) quotient out framework syntax differences. Moving a component from React to Svelte shares the exact same translation cache with **zero token re-spends** — the cost reduction this delivers scales with how much cross-framework duplication exists in a given repo, so we report it per-repo rather than as a single blanket percentage.
3. **Supported source surfaces**: Dedicated parsers cover JSX, Vue SFCs, and Svelte, alongside HTML, Markdown, and catalog formats. The `.py`, `.go`, `.rb`, `.php`, `.swift`, `.dart`, `.astro`, and `.ts` path in `src/parsers/code.rs:79` is one regex line-scanner, not a language parser. Android XML and Flutter ARB are catalog formats.
4. **Translation integrity checks (`IntegrityGate`)**: Tests cover placeholder changes, markup changes, and suspicious HTML. These are software tests, not a mathematical proof or a real-world guarantee.
5. **Zero Git Merge Conflicts (Monoidal Lockfile)**: `glossa.lock` and `lingo.lock` merge via commutative monoid fusion (`Table::fuse`). Two developers translating different features in parallel merge branches with a merge operation based on commutative monoid fusion.
6. **Zero-Runtime Type-Safe Codegen (`glossa codegen`)**: Compiles 0kB-overhead TypeScript and Rust message accessors with full autocomplete and compile-time slot type checking.
7. **In-Context Visual Studio (`glossa visual`)**: Local embedded web server on port 3920 allows developers and translators to Alt+Click any element in a browser to edit copy in real time without heavy Docker containers.
8. **Translation Memory (TM) Fuzzy Matching**: Built-in Levenshtein and 3-gram Jaccard similarity scoring ranks prior translations to prevent redundant LLM spending.
9. **Test suite**: 111 `#[test]` cases are present under `src/` and `tests/`. Fixtures are inline literals; there is no checked-in or cloned real-repository corpus, so these tests do not establish real-world recall or false-positive rates.

---

## Architecture

```
                                 [JSX / TSX] (React / Next.js / Solid)
                                 [Vue SFC]   (<template>)
                                 [Svelte]    (.svelte)
                                 [Catalogs]  (Android XML / Flutter ARB / JSON / YAML / PO)
   Raw Source Code Surfaces  ──► [Line scan] (.py .go .rb .php .swift .dart .astro .ts)
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

SPBench is a fixture micro-benchmark: it repeatedly parses hardcoded in-memory string literals, not files or a repository. The baseline column below is a fixture micro-benchmark vs `@babel/parser`; no competitor localization product was measured. The run uses inline synthetic literals and there is no real-repository corpus. CI has not been regenerating results successfully since 2026-09-25.

| Benchmark subsystem | Glossa fixture result | Baseline (fixture micro-benchmark vs @babel/parser) | Scope |
|---|---:|---|---|
| **JSX fixture parsing** | 17.58 µs per iteration | 84.42 µs per iteration | Hardcoded in-memory fixtures only; not a per-file or product comparison (4.8× fixture result) |

An earlier version reported a blanket **"2,300× faster"** figure. That compared Glossa latency with an assumed, never-measured ~15 ms/file incumbent cost. A fixture comparison with `@babel/parser` measured about 84 µs per fixture parse, not 15 ms. The earlier 2,300× figure was unsupported and has been retracted. The remaining 4.8× result is only this fixture micro-benchmark; it does not compare localization products. The 8.3 MB figure is peak RSS during the SPBench loop, which never opens a file. The 375 MB comparator came from `npx lingo.dev@latest --version`, a version-print command. Those workloads are not comparable, so no memory ratio is claimed.

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

FSL-1.1-ALv2 (Functional Source License, Version 1.1, Apache License 2.0 Change License). See [LICENSE](LICENSE). Copyright (c) 2026 Ptyktos Team.
