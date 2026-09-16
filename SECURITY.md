# Security Policy

## Supported versions

| Version | Supported          |
| ------- | ------------------ |
| 1.x     | :white_check_mark: |
| < 1.0   | :x:                |

Glossa is **private during development**. Security fixes land on `master` first and are
forwarded to supported release branches. There is no LTS policy yet.

## Reporting a vulnerability

**Do not open a public GitHub/GitLab issue for security reports.**

Email **`security@ptyktos.com`** with:

- Affected version (`glossa --version`) and platform
- Description of the vulnerability and its impact
- Steps to reproduce (minimal input + command)
- Any suggested fix or mitigation (optional but appreciated)
- Whether you want public credit, and under what name

### What happens next

1. Acknowledgement within **48 hours**.
2. Triage and severity assessment (CVSS-guided) within **5 business days**.
3. Fix developed privately; reporter gets a pre-release build to verify where feasible.
4. Coordinated disclosure: advisory published alongside the fixed release, reporter
   credited unless they opt out.

## Scope

In scope: the `glossa` binary and libraries in `src/`, the GitHub Action (`action.yml`),
lockfile parsers, and the `glossa visual` local server.

Out of scope (but appreciated if reported): the marketing site, example projects,
third-party translation provider integrations used only with user-supplied API keys.

## Hardening notes for reviewers

- `#![forbid(unsafe_code)]` across the workspace.
- No network calls in `extract` / `check` paths — translation providers are only
  contacted by explicit `glossa translate` invocations with user-supplied keys.
- `glossa visual` binds `127.0.0.1` by default; never `0.0.0.0` unless `--host` is
  passed explicitly.
- Lockfiles are parsed with size caps; see `src/` size-limit constants. Fuzz targets
  welcome — see CONTRIBUTING.md.
