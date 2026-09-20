# spbench real baselines

`run_spbench_suite()` in `../../src/cli/mod.rs` reports glossa's own speed
against incumbent baselines. Every `*_incumbent_us` constant in that
function is measured here — not assumed — using `npx lingo.dev@latest`'s
real peak RSS, real `@babel/parser`/`@vue/compiler-sfc` parse timing, a real
Node.js object-merge, a real regex-based slot/tag validator, and real
`JSON.stringify`/`js-yaml`/`gettext-parser` catalog emission, each on the
exact sample content or operation count glossa's own benchmark uses.

Re-run after bumping a dependency version, or periodically, to catch drift:

```bash
npm install
node benchmark1-extraction.mjs
node benchmark2-3-merge-and-gate.mjs
node benchmark4-emitters.mjs
/usr/bin/time -v npx --yes lingo.dev@latest --version  # Maximum resident set size
```

If a number moves meaningfully, update the matching `*_incumbent_us`
constant and its sourcing comment in `run_spbench_suite()`.

**Known gap:** `benchmark4-emitters.mjs` measures JSON/YAML/PO only (3 of
glossa's 4 formats) — a real `fluent.js` measurement wasn't taken rather
than fabricating a 4th data point. This makes that one baseline slightly
favorable to glossa (missing the incumbent's 4th-format cost). Fix by adding
a real fluent.js timing, not by estimating one.
