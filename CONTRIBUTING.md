# Contributing to Glossa

Thanks for your interest in Glossa. This document is the contract for getting changes merged.

## License reality (read first)

Glossa is **private during development** and licensed under **FSL-1.1-Apache**
(Functional Source License 1.1, Apache-2.0 future license). Production use requires a
commercial license until the Change Date, when the codebase converts automatically to
Apache-2.0. **By submitting a pull request you agree your contribution is licensed under
the same FSL-1.1-Apache terms.** If that is a problem for you, do not contribute.

## Primary contributor

**yokoszn** — architecture, review, and merge decisions. Tag `@yokoszn` on PRs that need
a decision. Silence for 7 days means "not prioritized", not "rejected" — re-ping once,
then move on.

## How to contribute

1. **Issues first for anything non-trivial.** Open an issue at
   `https://github.com/Ptyktos/glossa/issues` describing the problem, the proposed fix,
   and which of the 61 proof suites it touches. PRs without a linked issue may be closed
   unread if they touch `src/` core paths.
2. **Fork, branch, PR.** Branch from `master`. One concern per PR. Keep diffs reviewable
   (< 400 lines changed excluding generated files).
3. **Tests are mandatory.** New behavior ships with tests in `tests/`. Bug fixes ship with
   a regression test that fails before the fix and passes after. Run the full suite before
   pushing: `cargo test` must be green.
4. **Benchmarks for perf claims.** Any PR claiming a speedup includes `spbench` numbers
   (quality / speed / baseline on the same held-out data, per workspace rule REG-1).
   A remembered number is not a measurement.
5. **No generated files in PRs.** `glossa.lock`, `lingo.lock`, built binaries, and
   `target/` output stay out. CI regenerates what it needs.

## Code standards

- `cargo fmt --check` and `cargo clippy -- -D warnings` must pass.
- Zero-dependency discipline: new third-party crates need justification in the PR
  description. Prefer the workspace's existing primitives (`hashtrinity`, `bedrock`).
- `#![forbid(unsafe_code)]` is non-negotiable.
- Public API changes require a docs update in the same PR (`docs/` or README).

## Reporting issues

Use the issue templates (bug report / feature request). A good bug report includes:

- Glossa version (`glossa --version`) and platform (`uname -a`)
- Minimal reproduction: smallest input directory + command that triggers it
- Expected vs actual output, pasted verbatim
- `glossa.lock` snippet if the issue involves merge behavior

Security issues: **do not open a public issue.** Email `security@ptyktos.com` (see
SECURITY.md). General questions and commercial licensing: `glossa@ptyktos.com`.

## Commit messages

Conventional Commits: `feat:`, `fix:`, `docs:`, `test:`, `bench:`, `chore:`.
Scope the subsystem where useful: `feat(extract): support Svelte runes`.
