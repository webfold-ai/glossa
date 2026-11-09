# Glossa: Universal Sovereign Localization Engine
## Architectural Specification for Total Subsumption of Crowdin, Tolgee, and Lingo.dev

---

### Executive Summary

Glossa is the high-performance, sovereign localization compiler and continuous translation platform built on the Apeiron algebraic coordinate ring $R[x]/(x^2 - \sigma)$ and Hashtrinity monoid accumulator. Measured via the SPBench suite against real incumbent tooling, Glossa delivers a 4.8×–12.6× speedup (depending on subsystem) and ~45× lower memory footprint compared to legacy Node/Python-based localization stacks — see [README.md#empirical-benchmarks-spbench-suite](../README.md#empirical-benchmarks-spbench-suite) for the measured, reproducible numbers. (An earlier draft claimed "1,300× speedup and 85%+ lower operating costs" — that figure was never independently measured and did not survive being checked against a real `@babel/parser` baseline.)

This specification details the end-to-end architecture enabling Glossa to achieve **complete feature, format, ecosystem, and quality capability parity and subsumption** over:
1. **Crowdin & Crowdin Store**: Complete format matrix, all featured store integrations (Git, Figma, CMS, Zendesk, Intercom, Mailchimp, Notion), and enterprise translation memory.
2. **Tolgee**: Bi-directional in-context visual translation overlay SDK (Alt+Click on DOM elements), zero-reload translation updates, and open translation memory.
3. **Lingo.dev**: Automated translation quality scoring in unread languages (MTQE/MQM), customizable scorecards, root-cause error attribution (model vs. context vs. rule vs. glossary), longitudinal edit rate/distance tracking, rule auto-induction from human edits, and retrieval-augmented localization (RAL) with multi-LLM routing.

Licensed under **Apache-2.0**, Glossa offers zero-friction adoption for internal and commercial use alike.

---

## 1. System Architecture Overview

```
                          ┌────────────────────────────────────────────────────────┐
                          │                Ecosystem Ingress Ports                 │
                          │   Git (GH/GL/BB) │ CMS (Contentful/Strapi) │ Figma     │
                          │   Helpdesk (Zendesk/Intercom) │ Markdown / PO / XLIFF  │
                          └───────────────────────────┬────────────────────────────┘
                                                      │
                                                      ▼
                                       ┌─────────────────────────────┐
                                       │   Universal Parser Matrix   │
                                       │  Lossless AST Roundtripping │
                                       └──────────────┬──────────────┘
                                                      │
                                                      ▼
                                       ┌─────────────────────────────┐
                                       │  Retrieval-Augmented (RAL)  │
                                       │  Code AST Context Extractor │
                                       └──────────────┬──────────────┘
                                                      │
                         ┌────────────────────────────┴────────────────────────────┐
                         │                                                         │
                         ▼                                                         ▼
       ┌───────────────────────────────────┐                     ┌───────────────────────────────────┐
       │   Apeiron Coordinate TM Cache     │                     │      Multi-LLM Sovereign Hub      │
       │   R[x]/(x^2 - σ) O(1) Ring Store  │                     │ Claude 3.5 / GPT-4o / DeepSeek    │
       │   Sub-millisecond Vector Fallback │                     │   Dynamic Budget & Fallback Bus   │
       └─────────────────┬─────────────────┘                     └─────────────────┬─────────────────┘
                         │                                                         │
                         └────────────────────────────┬────────────────────────────┘
                                                      │
                                                      ▼
                                       ┌─────────────────────────────┐
                                       │ Morpho-Syntactic Glossary & │
                                       │   Terminology Enforcement   │
                                       └──────────────┬──────────────┘
                                                      │
                                                      ▼
                                       ┌─────────────────────────────┐
                                       │  Automated Quality Engine   │
                                       │  MQM/MTQE Scorecards (0-100)│
                                       │  Root-Cause Error Isolator  │
                                       └──────────────┬──────────────┘
                                                      │
                         ┌────────────────────────────┴────────────────────────────┐
                         │                                                         │
          [Score >= Threshold]                                     [Score < Threshold]
                         │                                                         │
                         ▼                                                         ▼
       ┌───────────────────────────────────┐                     ┌───────────────────────────────────┐
       │     Automated Release Pipeline    │                     │  Human-in-the-Loop Review Engine  │
       │  Continuous Daemon / Git PR Push  │                     │  1-Click Disambiguation Modal UI  │
       │  Headless CMS Cache Invalidation  │                     │  Rule Auto-Induction from Edits   │
       └─────────────────┬─────────────────┘                     └─────────────────┬─────────────────┘
                         │                                                         │
                         │                                                         ▼
                         │                                       ┌───────────────────────────────────┐
                         │                                       │   Lorelake Synthetic DPO Bridge   │
                         │                                       │   Harvest DPO Pairs for Training  │
                         │                                       └───────────────────────────────────┘
                         ▼
       ┌───────────────────────────────────┐
       │  In-Context Browser SDK Overlay   │
       │  Live Alt+Click DOM Editing       │
       └───────────────────────────────────┘
```

---

## 2. Milestone Alignment & Functional Modules

The implementation is structured across five Operately milestones under Project `glossa-universal-engine-Cb2OylAklTKdix7gaKABUj`:

| Milestone | Target Date | Core Subsumption Focus |
|---|---|---|
| **GM1: Universal Format & Ecosystem Integration Engine** | 2026-09-30 | Crowdin Store, Tolgee format matrix, Git/CMS/Design connectors |
| **GM2: Retrieval-Augmented Localization (RAL) & Multi-LLM Routing Hub** | 2026-10-07 | Lingo.dev RAL context engine, multi-model routing, Apeiron TM cache |
| **GM3: Automated Translation Quality Scoring (MTQE/MQM) & Error Tracing** | 2026-10-14 | Lingo.dev unread-language scoring, MQM scorecards, root-cause tracing |
| **GM4: Human-in-the-Loop Proofreading & Rule Auto-Induction Pipeline** | 2026-10-21 | Lingo.dev rule drafting from human edits, glossary mining, review modal |
| **GM5: Continuous Localization Daemon, Webhooks & In-Context Preview** | 2026-10-28 | Tolgee in-context browser overlay, continuous daemon, Lorelake DPO bridge |

---

## 3. Detailed Architectural Specifications

### 3.1 GM1: Universal Format & Ecosystem Integration Engine

#### 3.1.1 Universal Parser & Serializer Matrix
Glossa implements lossless, zero-copy Rust AST parsers and serializers for all internationalization formats in enterprise circulation:
- **XLIFF (XML Localization Interchange File Format)**: Complete support for XLIFF 1.2 and XLIFF 2.0/2.1 standards, preserving `<trans-unit>`, `<segment>`, state attributes (`needs-translation`, `final`), inline tags (`<g>`, `<x>`, `<ph>`), and tool annotations.
- **GNU gettext PO/POT**: Full bi-directional parsing of gettext catalog files, extracting `msgid`, `msgstr`, `msgid_plural`, `msgctxt` (disambiguation context), developer comments (`#.`), translator comments (`# `), and source references (`#: path:line`).
- **Mobile Native Formats**:
  - **Apple iOS/macOS**: Strings catalogs (`.xcstrings` JSON format introduced in Xcode 15) and legacy `.strings` / `.stringsdict` formats with pluralization and variable substitution.
  - **Android XML**: `res/values/strings.xml`, `plurals.xml`, and `arrays.xml` supporting formatted strings (`%1$s`), HTML entity escaping, and xliff namespaces.
  - **Flutter**: Application Resource Bundle (`.arb`) with ICU message format syntax and `@string_name` metadata blocks.
- **Web & Backend Catalogs**:
  - **JSON & Nested Catalogs**: Standard flat key-value JSON, nested JSON hierarchies, i18next format, and FormatJS/react-intl AST structures.
  - **YAML & Properties**: Ruby on Rails YAML, Symfony YAML, and Java `.properties` bundles.
  - **Structured Document Formats**: Markdown (`.md`) and MDX (`.mdx`) with YAML/TOML frontmatter preservation, code block isolation, and Markdown AST text-segment extraction.

#### 3.1.2 Git-Native Provider Sync & PR Automation Engine
Subsumes Crowdin and Tolgee Git integrations with complete support for GitHub, GitLab, Bitbucket, and Azure Repositories:
- **Bi-directional Change Detection**: Git tree SHA diffing identifies added, modified, and deleted source strings in commits and pull requests.
- **Automated Pull Request Dispatch**: Automatically creates PRs/MRs on target localization branches (`l10n/translations`, `i18n/[locale]`) containing formatted delta summaries, token statistics, and automated MQM score reports.
- **AST-Aware Merge Conflict Reconciliation**: Eliminates git merge conflicts on key-value catalogs by sorting and reconciling keys topologically rather than relying on raw line diffs.
- **Branch Protection & Webhook Verification**: Cryptographic HMAC signature validation on all incoming Git webhooks (`push`, `pull_request`, `merge_request`).

#### 3.1.3 Enterprise CMS & Design Connectors
Subsumes Crowdin Store top apps:
- **Headless CMS Connectors**:
  - Direct REST and GraphQL API adapters for **Contentful, Strapi, Sanity, Payload CMS, and WordPress**.
  - Automatically fetches localized schema fields, drafts localized entries, and respects publication workflows (Draft -> Review -> Published).
- **Bidirectional Figma Design Plugin**:
  - Extracts text layers from Figma design canvases with frame context and bounding-box coordinates.
  - Simulates target language string expansion (e.g. German text +35% longer than English) and computes visual overflow warnings inside Figma canvas before engineering implementation begins.

#### 3.1.4 Customer Comms & Help Desk Adapters
- **Customer Support Knowledge Bases**: Live bi-directional sync with **Zendesk Guide** and **Intercom Help Center**, translating articles, help categories, and macro response snippets while preserving HTML callouts.
- **Marketing Automation**: Connectors for **Mailchimp** newsletter templates and **HubSpot** email/landing page assets.
- **Knowledge Bases**: Ingress from **Notion** documentation workspaces.

---

### 3.2 GM2: Retrieval-Augmented Localization (RAL) & Multi-LLM Routing Hub

#### 3.2.1 Retrieval-Augmented Localization (RAL) Code AST Context Engine
Subsumes Lingo.dev's core innovation: eliminating out-of-context translation errors by providing rich surrounding code context.
- **Syntactic Context Harvesting**:
  - Glossa inspects the source code AST surrounding the localized string:
    - **React/Next.js (TSX/JSX)**: Identifies enclosing component name (`<Button>`, `<NavigationModal>`, `<BillingHeader>`), sibling text nodes, parent containers, and props.
    - **Flutter/Dart**: Traverses Widget tree context (`AppBar`, `Tooltip`, `SnackBar`).
    - **Vue/Svelte**: Parses template hierarchy, directives (`v-text`, `v-bind:alt`), and slot boundaries.
  - **Semantic Context Augmentation**: Extracts nearby source code comments (`// i18n-hint:`, `/* Context: Admin role only */`), file path semantics (`apps/admin/billing/invoices.tsx`), and variable bindings.
- **Zero-Latency In-Memory Context Injection**: Formats structural context into the prompt payload to eliminate polysemic ambiguity (e.g., distinguishing "Book" as a noun vs. verb, "Close" as an action vs. adjective).

#### 3.2.2 Multi-LLM Sovereign Routing Hub
Provides vendor-agnostic routing across commercial and self-hosted models:
- **Commercial LLM Adapters**: Native streaming connectors for:
  - **Anthropic Claude 3.5 Sonnet / Haiku** (superior syntactic reasoning and nuanced tone).
  - **OpenAI GPT-4o / GPT-4o-mini** (high throughput, structured output enforcement).
  - **Google Gemini 1.5 Pro / Flash** (massive context window for whole-document translation).
  - **Mistral Large / Codestral** (European data sovereignty compliance).
- **Self-Hosted Zero-Retention Inference**: Direct OpenAI-compatible API connector for **Ollama, vLLM, and TGI** hosting open models (**DeepSeek V3/R1, Llama 3.3 70B, Qwen 2.5**), ensuring 100% air-gapped, zero-data-retention compliance for enterprise deployments.
- **Intelligent Budget & Fallback Bus**:
  - Dynamically routes strings based on complexity: simple short phrases route to ultra-fast low-cost models; complex marketing copy routes to high-reasoning models.
  - Automatically fails over to alternate providers on HTTP 429 (rate limits) or 5xx server errors with zero dropped requests.

#### 3.2.3 Morpho-Syntactic Glossary & Terminology Enforcement
- **Morphological Inflection Engine**:
  - Prevents naive exact-string substitution that breaks target grammar.
  - Injects grammatical constraints: case declension (nominative, accusative, dative, genitive in German/Slavic languages), grammatical gender agreement (masculine/feminine/neuter in Romance languages), and plural categories (ICU `zero`, `one`, `two`, `few`, `many`, `other`).
- **Negative Glossaries & Brand Voice Constraints**: Enforces strictly prohibited words, competitor trademarks, and brand guidelines (e.g., informal *du* vs. formal *Sie* in German).
- **Locked Token Protection**: Guarantees untranslatable parameters (`{username}`, `{count}`, `<Link>`, `{{tier_price}}`) are never translated, corrupted, or omitted.

#### 3.2.4 Apeiron Algebraic Translation Memory & Coordinate Cache
- **Mathematical Foundation**: Leverages the Apeiron coordinate ring $R[x]/(x^2 - \sigma)$ and Hashtrinity monoid accumulator to index all historical source-target pairs as deterministic algebraic coordinates.
- **Performance Characteristics**:
  - $O(1)$ exact coordinate match lookup in sub-microsecond time.
  - Sub-millisecond vector similarity fuzzy-match fallback (leveraging SIMD-accelerated cosine distance).
  - Cross-project and cross-branch deduplication: Identical strings across repositories incur **zero recurring LLM token expenditures**.

---

### 3.3 GM3: Automated Translation Quality Scoring (MTQE/MQM) & Error Tracing

#### 3.3.1 Multidimensional MQM/MTQE Scorecard Engine
Subsumes Lingo.dev's quality scoring in languages product managers and engineers cannot read:
- **Multidimensional Quality Metrics (MQM) Framework**: Evaluates translations on a normalized scale from **0 to 100** across four foundational dimensions:
  1. **Accuracy**: Faithfulness to source meaning, zero hallucinations, zero omissions, correct preservation of numerical values, dates, and placeholders.
  2. **Fluency**: Natural syntax, grammatical correctness, natural idiomatic expressions, correct spelling and punctuation according to locale conventions.
  3. **Brand Voice & Style**: Adherence to defined persona guidelines (e.g. professional, playful, concise, authoritative), correct formality register.
  4. **Terminology**: Strict adherence to project glossaries and correct handling of technical domain nomenclature.
- **Customizable Scorecard Weightings**: Teams can define custom weights (e.g. Legal/Finance catalogs: 50% Accuracy, 30% Terminology, 10% Fluency, 10% Voice; Marketing landing pages: 40% Voice, 30% Fluency, 20% Accuracy, 10% Terminology).
- **Target Language Evaluation in Unread Languages**: The automated evaluator runs native linguistic checks on complex character sets and morphologically rich languages (Japanese, Chinese, Korean, Arabic, Hebrew, Finnish, Hungarian) without requiring human speaker intervention.

#### 3.3.2 Root-Cause Error Attribution Engine
When a translation scores below the desired quality threshold, Glossa does not just flag it as low quality—it runs an attribution classifier that isolates the exact cause into one of four root categories:
1. **Model Error**: The underlying LLM exhibited hallucination, truncated content, grammatical distortion, or missed nuance despite adequate prompt instructions.
2. **Context Deficiency**: The source string was ambiguous because the surrounding AST context, component hierarchy, or developer comments were missing or insufficiently descriptive.
3. **Translation Rule Violation**: The output violated an explicit project translation rule (e.g., exceeding maximum character width for mobile buttons, improper formality).
4. **Missing Glossary Term**: A domain-specific noun, brand entity, or product feature was translated inconsistently because it was absent from the project glossary.

#### 3.3.3 Temporal Translation Edit Rate (TER) & Levenshtein Analytics
Tracks the reduction of human post-editing effort over time:
- **Metrics Tracked**:
  - **TER (Translation Edit Rate)**: The number of edits (insertions, deletions, substitutions, word shifts) required to normalize the machine output divided by the total number of words.
  - **Normalized Levenshtein Character & Token Distance**: Measuring character-level edit distance between machine output and approved human output.
  - **Human Touch Rate**: Percentage of strings requiring human modification before publication.
- **Continuous Improvement Proof**: Provides visual analytics proving that as the Apeiron TM and glossary grow, human edit rates steadily decline toward zero.

#### 3.3.4 Unread-Language Quality Gating & Deployment Blocker
- **Automated Release Policy**: Configurable threshold gates for CI/CD and deployment pipelines (e.g., "Block production release if Japanese composite MQM score < 85/100 or if any Critical Accuracy error exists").
- **Audit Logging**: Generates cryptographically verifiable quality certificates for enterprise compliance audits.

---

### 3.4 GM4: Human-in-the-Loop Proofreading & Rule Auto-Induction Pipeline

#### 3.4.1 In-Context Disambiguation Modal UI
Provides a high-velocity review interface for internal translators, native speakers, and agency reviewers:
- Displays source text, surrounding DOM hierarchy, UI component screenshot, and candidate translations.
- Multi-Model Comparison: Displays translations from multiple candidate models side-by-side with individual MQM sub-scores.
- 1-Click Operations: One-click approval, one-click replacement, and instant diff preview.

#### 3.4.2 Rule Auto-Induction from Human Post-Edits
Subsumes Lingo.dev's rule auto-generation capability:
- **Diff Analysis**: When a human reviewer edits a translation, Glossa's semantic induction engine analyzes the before-and-after diff.
- **Rule Formulation**: Automatically derives generalized, reusable translation rules (e.g., *"In Japanese, render button CTAs using polite imperative form (-te kudasai) rather than dictionary form"* or *"Always translate 'Workspace' as 'Arbeitsbereich' in German, never as 'Werkraum'"*).
- **Rule Registry Updates**: Rules are automatically drafted and submitted to the project rule registry for team review and continuous model steering.

#### 3.4.3 Active Term Candidate Mining & Dynamic Glossary Expansion
- Scans human edit streams to identify recurring terms, brand entities, and specialized jargon not currently indexed in the glossary.
- Prompts reviewers with 1-click suggestions: *"You translated 'Tenant' as 'Mandant' 14 times. Add to project glossary?"*
- Upon confirmation, updates the Apeiron terminology store and retroactively checks pending translations.

#### 3.4.4 Role-Based Locale Team Assignment & Workflow Routing
- Granular permission matrices dividing responsibilities: Developers (source strings), Native Translators (first pass), QA Proofreaders (final approval).
- Language-specific assignment: Specific reviewers routed exclusively to their verified language pairs.
- Automated notification integrations for Slack, Discord, and email.

---

### 3.5 GM5: Continuous Localization Daemon, Webhooks & In-Context Preview

#### 3.5.1 Zero-Latency Continuous Daemon (`glossa-daemon`)
- Low-overhead background process written in Rust utilizing `notify` / `inotify` kernel events.
- Watches project source directories for file modifications.
- Executes sub-millisecond AST diffing: detects exact string additions and edits, runs incremental translations through Apeiron TM / LLM hub, and updates locale catalogs in real time with zero developer interruption.

#### 3.5.2 Live In-Context In-Browser Overlay Translation SDK
Subsumes Tolgee's in-context visual translation interface:
- **Lightweight Browser SDK**: Drop-in script or React/Vue wrapper (`@glossa/web`).
- **Alt+Click In-Context Translation**: While browsing the running web application in development or staging, holding `Alt` (or `Option`) and clicking on any translated text opens a floating in-context editor modal.
- **Live DOM Mutation**: Edits made in the overlay instantly update the running DOM in real time and push the modified translation back to the Glossa daemon and Apeiron coordinate store without restarting the dev server.
- **Automatic Screenshot Capture**: Automatically captures the viewport bounding box of the element and attaches it as visual context for other reviewers.

#### 3.5.3 Multi-Platform Webhook Dispatcher & CMS Invalidation Bus
- Dispatches event-driven webhooks on string lifecycle triggers: `string.created`, `score.evaluated`, `translation.approved`, `bundle.exported`.
- Integrates directly with frontend CDN cache invalidation APIs (Cloudflare, Vercel, AWS CloudFront) and headless CMS publication webhooks.

#### 3.5.4 Synthetic Localization DPO & Preference Pair Exporter to Lorelake
- **Data Harvesting Flywheel**: Every translation interaction produces valuable behavioral signal:
  - **DPO / RLHF Pairs**: Source string + Context + Model Output (Rejected) vs. Human Post-Edit (Chosen).
  - **Quality Score Ground Truth**: Translations rated alongside granular MQM error sub-category breakdowns.
- **Sanitization & Federation**: Harvested interaction pairs are routed through `pii-svc` for in-memory anonymization and streamed to `lorelake` / `sangdb` over heterogeneous S3 storage (`rustfs` / `garage`).
- **Commercial Monetization**: Curated localization preference datasets are made available via `lorelake-hf-bridge` for downstream LLM translation fine-tuning and enterprise data licensing.

---

## 4. Verification & Implementation Verification Matrix

| Subsumed Capability | Incumbent Benchmark | Glossa Implementation | Verification Status |
|---|---|---|---|
| Complete Catalog Formats | Crowdin / Tolgee Matrix | Native Rust AST Parsers (`src/formats/`) | GM1 Verified |
| Git & CMS Providers | Crowdin Store Apps | `glossa-git` & `glossa-cms` adapters | GM1 In Progress |
| In-Context Browser Overlay | Tolgee Web Overlay | `@glossa/web` Alt+Click DOM SDK | GM5 In Progress |
| Retrieval-Augmented Localization | Lingo.dev RAL Engine | Surround AST Context Extractor (`src/ral/`) | GM2 In Progress |
| Multi-LLM Routing & Fallback | Lingo.dev LLM Wrapper | Pluggable Provider Hub (`src/llm/`) | GM2 In Progress |
| Unread-Language Quality Scoring | Lingo.dev MTQE Scorecard | Multidimensional MQM Evaluator (`src/mqm/`) | GM3 In Progress |
| Root-Cause Error Attribution | Lingo.dev Error Tracing | 4-Class Diagnostic Classifier (`src/audit/`) | GM3 In Progress |
| Edit Rate / Distance Reduction | Lingo.dev TER Metrics | Temporal TER & Levenshtein Bus (`src/stats/`)| GM3 In Progress |
| Rule Auto-Induction from Edits | Lingo.dev Rule Drafting | Semantic Diff Rule Inducer (`src/rules/`) | GM4 In Progress |
| Continuous Translation Daemon | CLI Watchers | Rust-native `glossa-daemon` (`src/bin/`) | GM5 In Progress |
| Monetizable Training Data Export| Proprietary Data Silos | Lorelake HuggingFace Hub Bridge | GM5 In Progress |
