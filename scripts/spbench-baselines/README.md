# Node.js timing scripts

Standalone Node.js scripts that time other tools' code on the same kinds of input that
`glossa bench` uses: `@babel/parser` and `@vue/compiler-sfc` for extraction, a plain object merge, a
regex slot/tag validator, and `JSON.stringify` / `js-yaml` / `gettext-parser` for catalog output.

`glossa bench` does not read or compare against these scripts, and no results are published with
this repository. Numbers from either side depend on the machine, the Node.js and Rust versions and
the input, so only compare runs you make yourself on the same machine.

```bash
npm install
node benchmark1-extraction.mjs
node benchmark2-3-merge-and-gate.mjs
node benchmark4-emitters.mjs
cargo run --release --bin glossa -- bench
```
