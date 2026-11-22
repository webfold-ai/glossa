# Contributing

Issues and pull requests are welcome for bug fixes, extraction support for more frameworks and
languages, translation provider integrations, and documentation.

## Before opening a pull request

- Search existing issues and pull requests; describe the user-visible problem and expected behavior.
- Keep changes focused. Add or update a test when behavior changes.
- Do not include real API keys or a customer's source code in commits or issue attachments. Use
  synthetic fixtures.
- For provider integrations (`src/engine/providers/`), state which provider you tested against and
  whether you used a real API key or a mock.
- Do not add performance or comparison numbers to the docs unless you re-measured them and state the
  machine, date and command.

## Development setup

```sh
cargo build
cargo test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

Run the CLI locally with `cargo run --bin glossa -- <command>`. The `lingo` binary is the same CLI
under a second name.

## Pull requests

Open a pull request against `master`. Include a concise summary, the commands you ran and their
outcomes, and any security or compatibility impact.

## Security reports

Do not report vulnerabilities in public issues or pull requests. Follow [SECURITY.md](SECURITY.md).

## License

By contributing, you agree that your contribution is offered under the repository's Apache-2.0 license.
