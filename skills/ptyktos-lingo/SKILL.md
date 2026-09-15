---
name: ptyktos-lingo
description: 2,300x faster, 85%+ cheaper universal multi-framework localization compiler and CI/CD translation engine. Automates string extraction, AI translation with Integrity Gate verification, lockfile monoid merging, and CI checks.
---

# ptyktos-lingo Agent Skill

Use this skill when localizing web applications (React/Next.js, Vue/Nuxt, Svelte, Angular, Solid, Astro), mobile apps (iOS Swift, Android Kotlin, Flutter Dart), backend services (Python, Go, Ruby, PHP), or enterprise catalogs (JSON, YAML, PO, Fluent, Properties, XLIFF, ARB).

## The 3-Liner Project Integration

To immediately automate localization in any repository:

```bash
# 1. Install or download binary
curl -fsSL https://ptyktos.com/install.sh | bash

# 2. Initialize project (auto-detects React, Vue, Svelte, Flutter, Python, Go, etc.)
lingo init --locales es,fr,de,ja

# 3. Extract & verify coverage in CI
lingo extract && lingo check --ci
```

## Available CLI Commands

| Command | Purpose |
|---|---|
| `lingo init` | Auto-detects framework, generates `lingo.json` |
| `lingo extract` | Universal single-pass sweep extracting all translatable strings into canonical IR (`CanonicalTerm`) |
| `lingo translate` | Translates missing keys via Anthropic, OpenAI, DeepSeek, Ollama, or Offline provider with Integrity Gate |
| `lingo check --ci` | CI check: emits GitHub error annotations (`::error ...::`), generates `$GITHUB_STEP_SUMMARY`, exits 1 if keys missing |
| `lingo diff` | Zero-network translation diff showing added, updated, and missing strings |
| `lingo pseudo` | Generates pseudo-localized strings with +35% expansion to test UI clipping before paying for LLM tokens |
| `lingo sync` | Synchronizes `lingo.lock` into target formats (`.json`, `.yml`, `.po`, `.ftl`, `.xml`, `.strings`, `.arb`, `.xlf`) |
| `lingo bench --spbench` | Runs SPBench performance micro-benchmarks with p50/p90/p99 percentiles and 3-column verification |
| `lingo mcp` | Starts JSON-RPC 2.0 Model Context Protocol stdio server for Cursor, Copilot, and Claude Code |

## Invariant Safeguards

- **Zero-Fault Integrity Gate**: All translations MUST conserve variable slots (`{user}`, `{{count}}`) and numbered markup tags (`<0>...</0>`). If an LLM drops a placeholder or injects unsafe HTML, the gate rejects the item immediately.
- **Zero Git Merge Conflicts**: `lingo.lock` merges via monoidal commutativity (`Table::fuse`), eliminating Git merge conflicts across parallel feature branches.
