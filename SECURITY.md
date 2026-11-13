# Security policy

## Report a vulnerability

Email **security@webfold.ai** to report a suspected vulnerability. Do not
open a public issue for an unpatched vulnerability. Include the affected
version or commit, impact, and steps to reproduce.

Once GitHub private vulnerability reporting is enabled in repository
settings, reports can also be submitted through
[GitHub Security Advisories](https://github.com/webfold-ai/glossa/security/advisories/new).

## Supported versions

Pre-1.0: security fixes are made against the latest `main` branch only.

## Security-relevant behavior

`glossa extract`/`glossa check` reads source files across a codebase you
point it at — arbitrary, potentially untrusted source code is its normal
input. Treat parser bugs here the same as any other code that parses
attacker-influenced input.

`glossa translate` sends extracted strings to whichever AI provider you
configure (`anthropic`, `openai`, `deepseek`, `ollama`, `ptyktos-api`, or
`offline`) — this is a real outbound network call carrying source-derived
text, authenticated with an API key you supply (`src/config/mod.rs`,
`src/engine/providers/`). Don't point it at a provider you don't want to see
your source strings, and don't commit the API key alongside project config.

`glossa visual` starts a local web server bound to `127.0.0.1` only
(`src/visual/mod.rs`), default port 3920 — it never binds a non-loopback
address, so it isn't reachable off the local machine by default.

`glossa.lock`/`lingo.lock` merges (`src/lockfile/`) are a commutative
monoid fuse over untrusted lockfile content from every branch being merged —
malformed lockfile input should fail the merge, not silently corrupt state.

## Dependency and release security

Rust dependencies are recorded in `Cargo.lock`. Pull requests and scheduled
workflows run RustSec advisory checks (`cargo audit`), cargo-deny policy
checks, and SBOM generation (see `.github/workflows/supply-chain.yml`).
