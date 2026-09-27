# Contributing

Thanks for considering a contribution. Issues and pull requests are welcome
for bug fixes, new framework/language extraction support, translation
provider integrations, and documentation.

## Before opening a pull request

- Search existing issues and pull requests; describe the user-visible
  problem and expected behavior.
- Keep changes focused. Include or update a regression test when behavior
  changes — this repo has a real benchmark suite (`glossa bench --spbench`)
  that CI regenerates fresh rather than trusting a hand-edited number; don't
  hand-edit `spbench-results.json` or similar.
- Never include real API keys or a customer's actual source code in commits
  or issue attachments. Use synthetic fixtures.
- For provider integrations (`src/engine/providers/`), state which provider
  you tested against and whether you used a real API key or a mock.

## Development setup

```sh
cargo build
cargo test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

Run the CLI locally with `cargo run --bin glossa -- <command>` (also
available as the `lingo`-aliased binary, for drop-in compatibility with
existing `lingo.json`/`lingo.lock` projects).

## Pull requests

Open a pull request against `main`. Include a concise summary, the commands
run and their outcomes, and any security or compatibility impact. CI must
pass before merge.

## Security reports

Do not report vulnerabilities in public issues or pull requests. Follow
[SECURITY.md](SECURITY.md) for private reporting.

## License

By contributing, you agree that your contribution is offered under the
repository's FSL-1.1-ALv2 license.
