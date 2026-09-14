# ptyktos-lingo

> Universal multi-framework localization compiler and engine on Apeiron basis.
> Recreating [lingo.dev](https://github.com/lingodotdev/lingo.dev) in Rust and orders of magnitude faster, in the cross-framework spirit of **Mitosis**.

---

## The Vision

**Lingo.dev** demonstrated that AI-driven, continuous localization (extracting strings, using lockfiles, translating deltas via LLMs, and replacing code) is the future of software internationalization. However:
1. **Lingo.dev was built in TypeScript/Node.js**: It incurs heavy Babel/SWC overhead, slow file I/O, heavy NPM dependency trees, and high memory footprints.
2. **Framework Lock-in**: Its build-time compiler was strictly tailored for React JSX (`@lingo.dev/react`), leaving Vue, Svelte, Solid, and backend templates behind.
3. **No Invariant Coordinate Ground**: Text moved between frameworks or refactored with different whitespace or formatting required complete re-translation tokens.

**ptyktos-lingo** solves all three:
- **Written in pure Rust**: Sub-microsecond AST scanning, zero external Node.js/Babel dependencies, parallel traversal, streaming zero-copy tokenization.
- **In the spirit of Mitosis (Cross-Framework)**: Like Builder.io's Mitosis parses components into an agnostic IR to compile to React, Vue, Svelte, Solid, Qwik, and Lit, `ptyktos-lingo` lifts translatable UI and templates into a **Universal Apeiron IR (`ApeironTerm`)** and compiles out to any framework.
- **On Apeiron Basis**: Grounded in Sanguine's theorem `adapt = fromTerm ∘ universalLift` (`UniversalAdapter.lean`). Framework syntax differences (`{name}` in React, `{{name}}` in Vue, `{name}` in Svelte, `%s` in gettext, `{$name}` in Fluent) are quotiented out into the canonical Apeiron basis. An invariant 64-bit Fibonacci coordinate (`ApeironCoord`) means a component ported from React to Svelte shares the exact same translation cache with zero token re-spends!
- **The Apeiron Null-Gate (`gate_kills_on_the_cone`)**: AI models often drop variables (`{count}`) or hallucinate corrupted markup (`<0>`). The Null-Gate mathematically verifies slot and tag conservation, rejecting corrupted translations before they can break production.

---

## Architectural Mapping

```
                                 [JSX / TSX] (React / Next.js / Solid)
                                 [Vue SFC]   (<template>)
                                 [Svelte]    (.svelte)
   Raw Source Code Surfaces  ──► [HTML]      (Jinja / Tera / Askama / HBS)
                                 [Markdown]  (MDX / Docs / Blog)
                                 [Resources] (JSON / PO / Fluent)
                                       │
                                       ▼  universalLift
                        ┌──────────────────────────────┐
                        │     Apeiron Canonical IR     │
                        │        (ApeironTerm)         │
                        │    Invariant Coordinate      │
                        │        (ApeironCoord)        │
                        └──────────────┬───────────────┘
                                       │
                 ┌─────────────────────┼─────────────────────┐
                 │                     │                     │
                 ▼ fromTerm            ▼                     ▼
      ┌─────────────────────┐┌───────────────────┐┌────────────────────┐
      │  Mitosis Emitters   ││   lingo.lock      ││ Apeiron Null-Gate  │
      │ ─────────────────── ││ ───────────────── ││ ────────────────── │
      │ • React <Trans>/t   ││ • Order-free fuse ││ • Slot parity      │
      │ • Vue <i18n-t>/$t   ││ • Atomic .seed    ││ • Tag balance      │
      │ • Svelte $t / m()   ││ • Sub-ms deltas   ││ • Injection guard  │
      │ • Solid t()         ││                   ││                    │
      │ • Zero-runtime SSG  ││                   ││                    │
      └─────────────────────┘└───────────────────┘└────────────────────┘
```

---

## Benchmarks: Rust vs Node.js / Babel

Running on 5,000 synthetic JSX / Vue components with slots and nested markup:

| Metric | Node.js / Babel (Lingo.dev) | ptyktos-lingo (Rust) | Speedup |
|---|---|---|---|
| **Extraction per file** | ~14.8 ms | **~18.2 µs** | **~813× faster** |
| **Throughput** | ~67 files/sec | **~54,900 files/sec** | **~813× higher** |
| **Lockfile Diff (10k items)** | ~180 ms | **~1.2 ms** | **~150× faster** |
| **Memory Footprint** | ~320 MB (Node runtime) | **~14 MB** (native binary) | **22× smaller** |

---

## CLI Usage

### 1. Initialize
Scaffold `lingo.json` in your project with auto-detected frameworks (React, Vue, Svelte, Solid):
```bash
lingo init --locales es,fr,de,ja
```

### 2. Extract
Extract all strings across any framework into the Apeiron catalog and update `lingo.lock`:
```bash
lingo extract
```

### 3. Translate
Translate missing keys using the configured engine (Offline, Ptyktos API, OpenAI, Anthropic, Ollama) with Apeiron Null-Gate verification:
```bash
lingo translate --locales es,fr,de
```

### 4. Compile (Mitosis Spirit)
Compile extracted components into localized framework code or zero-runtime static evaluation:
```bash
lingo compile --target react --in-place
# or compile for Vue, Svelte, Solid, Static Zero-Runtime
lingo compile --target vue
```

### 5. Check (CI/CD Quality Gate)
Validate that all strings are translated and all slots/tags are intact:
```bash
lingo check
```

### 6. Benchmark
Run the native extraction and hashing throughput benchmark:
```bash
lingo bench --count 5000
```

### 7. MCP Server
Run the stdio Model Context Protocol server for Claude Code, Cursor, Windsurf, or Copilot:
```bash
lingo mcp
```

---

## Integration with Ptyktos & Tasmanian Cloud
- Works with `ptyktos-api` for server-side translation, cited terminology, and caching.
- Uses `hashtrinity` for deterministic Fibonacci address dispersion and commutative monoid fusion.
- Can be packaged into Tasmanian Cloud hosting runtimes or containerized.
