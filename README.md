# Glossa

Glossa is a localization tool written in Rust. It extracts translatable strings from source code
and catalog files, keeps them in a lockfile, translates missing entries through a provider you
configure, checks each translation with an integrity gate, and writes locale catalogs. It installs two
binaries, `glossa` and `lingo` (the same CLI under two names).

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](LICENSE)

## Quickstart

```bash
# Install (requires a Rust toolchain)
cargo install --git https://github.com/webfold-ai/glossa glossa

# Initialize: detects the framework and imports settings from other i18n tools' config files
glossa init --locales es,fr,de,ja

# Extract strings into the lockfile and check coverage
glossa extract && glossa check --ci
```

If a project already has a `lingo.json` / `lingo.lock`, Glossa reads and updates those files. The
lockfile is `glossa.lock` when it exists, otherwise `lingo.lock`. `glossa import` reads locale
settings from Lingo.dev (`i18n.json`, `lingo.json`), Inlang, Tolgee, Crowdin, Lokalise and i18next
config files; importing copies the settings it recognises (source and target locales, source
patterns, format) and does not migrate translation data.

## What it does

- **Extraction.** Extractors for React/JSX/TSX, Vue single-file components, Svelte, HTML-like
  templates (`.html`, `.htm`, `.tera`, `.jinja`, `.hbs`, `.ejs`), Markdown/MDX, translation-call
  patterns (`t()`, `$t()`, `_()`, `gettext`, `NSLocalizedString`, `String(localized:)`, `$localize`,
  and similar) in Python, Go, Ruby, PHP, Swift, Dart, Angular and Astro sources, and catalogs in
  JSON, YAML, PO, Fluent, Android XML, iOS `.strings`, Flutter ARB, Properties, XLIFF and CSV.
  Extraction is pattern-based; check the result on your own code.
- **Content coordinates.** Each translatable string gets a deterministic 64-bit coordinate derived
  from its content, so the same text found in a React file and a Vue file is one lockfile entry.
- **Lockfile.** `glossa.lock` / `lingo.lock` stores entries and per-locale translations. Merging two
  lockfiles with `Table::fuse` does not depend on order.
- **Translation.** `glossa translate` sends missing entries to `anthropic`, `openai`, `deepseek`,
  `ollama` or the built-in rule-based `offline` provider (for development and tests). API keys come
  from the environment (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, and so on).
- **Integrity gate.** Every translation is checked before it is stored: `{slots}` must match the
  source, numbered tags (`<0>...</0>`) must be balanced and in order, script and event-handler markup
  is rejected. The library also offers a glossary check (`ApeironNullGate::validate_with_glossary`);
  the CLI does not apply the config's `glossary` field yet.
- **PII placeholders.** Emails, phone numbers, IP addresses and card numbers matched by the patterns
  in `src/engine/pii.rs` are replaced with placeholders before text is sent to a provider and
  restored afterwards.
- **Translation memory (library module).** `src/engine/tm.rs` scores similarity between strings
  (Levenshtein and 3-gram Jaccard); the CLI does not use it yet.
- **Catalog output.** `glossa sync` writes JSON, YAML, PO, Fluent, Android XML, iOS `.strings`,
  Flutter ARB, Properties, XLIFF or CSV, depending on `output_format` in `glossa.json`.
- **Code generation.** `glossa codegen` writes TypeScript or Rust message accessors with typed slot
  parameters and no external runtime imports.
- **Visual editor.** `glossa visual` serves an in-context editor on `127.0.0.1:3920`.
- **MCP server.** `glossa mcp` serves four tools over stdio (see
  [docs/AI_WORKFLOWS_GUIDE.md](docs/AI_WORKFLOWS_GUIDE.md)).

## Commands

```bash
glossa init --locales es,fr,de,ja          # write glossa.json
glossa extract                             # scan sources, update the lockfile
glossa translate --provider anthropic --locales es,fr,de   # providers: anthropic, openai, deepseek, ollama, offline
glossa check --ci --format=github          # exit code 1 if translations are missing; prints ::error annotations
glossa diff                                # pending translations, no network
glossa pseudo --locale pseudo              # pseudo-localized strings for layout testing
glossa sync                                # write locale catalogs from the lockfile
glossa codegen --target typescript --output src/i18n/messages.ts
glossa codegen --target rust --output src/i18n.rs
glossa visual --port 3920
glossa import                              # import locale settings from other tools' config files
glossa export --locale es --format jsonl   # JSONL or TSV export of translations
glossa mcp
glossa lorelake --action push --bucket translations   # push lockfile entries to an HTTP object store
glossa bench                               # time Glossa's own code on this machine
```

`glossa compile` is accepted by the CLI but not implemented yet; it exits with an error. Use
`glossa codegen` for generated accessors. `glossa lorelake` supports only `--action push`.

Run `glossa --help` and `glossa <command> --help` for all options.

## Benchmarks

`glossa bench` times Glossa's extraction, lockfile, integrity-gate and emitter code on fixed
in-process samples and prints the timings it measured (add `--json-output <file>` for JSON). It does
not run or compare against any other tool, and no results are published here: timings depend on the
machine and build. [scripts/spbench-baselines/](scripts/spbench-baselines/) has Node.js scripts that
time other libraries on comparable operations, if you want to make your own comparison.

## Tests

`cargo test` runs unit tests and the integration suites in `tests/`:

- `regression_suite.rs`: extraction fixtures per ecosystem (React, Vue, Svelte, Angular, Astro,
  Python, Go, Ruby, PHP, Swift, Android XML, iOS strings, Flutter ARB, YAML, XLIFF, Properties, PO,
  Fluent) and the integrity gate.
- `false_positive_suite.rs`: code, CSS classes and URLs are not extracted; roundtrip through the
  catalog formats; lockfile merge in several orders; gate rejections; generated code.
- `ci_runner_test.rs`: `glossa check --ci` exit codes, annotations and step summary.

These test fixed fixtures; they do not establish behavior on code they do not cover.

## Documentation

- [Using Glossa with AI coding assistants (MCP and CLI workflow)](docs/AI_WORKFLOWS_GUIDE.md)
- [Contributing](CONTRIBUTING.md)
- [Security policy](SECURITY.md)

## License

Apache-2.0. See [LICENSE](LICENSE).
