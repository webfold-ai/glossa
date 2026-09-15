# Open-Source (OSS) Maintainer & Contributor Guide

> **Target Audience:** Open-Source Maintainers, Community Contributors, Foundation Stewards.  
> **Objective:** Deliver continuous, multi-language localization across your open-source project with $0 SaaS fees, zero CI compute drain, and zero Git merge conflicts.

---

## 1. Why Open-Source Projects Struggle with i18n

Internationalization is one of the most abandoned features in open-source software:
1. **The Contributor Dilemma:** A community contributor submits a great feature with two new buttons. They speak English, but your project supports Spanish, French, German, and Japanese. The PR sits unmerged for weeks waiting for translators to update JSON catalogs.
2. **The Git Merge Conflict Hell:** When three contributors touch localization catalogs simultaneously, Git reports merge conflicts (`<<<<<<< HEAD`). Maintainers waste hours manually rebasing translation files.
3. **The CI Minute Drain:** Tools like Babel or Node.js-based extractors take 20–45 seconds per run, quickly consuming free GitHub Actions quotas on active repositories.
4. **SaaS Vendor Friction:** Proprietary platforms like Crowdin or Lokalise require managing external seats, dealing with tier limits, and wrestling with noisy PR bots that clutter commit logs.

---

## 2. The Glossa Solution for Open Source

Glossa was designed to remove 100% of the friction from open-source localization:

* **$0 SaaS Overhead (Open Core):** 100% free and open-source under MIT / Apache 2.0. No per-seat fees, no key limits, no forced cloud subscriptions.
* **Sub-Second CI Execution:** Scans codebases in under 100 milliseconds, consuming virtually zero GitHub Actions minutes.
* **0 Git Merge Conflicts:** `glossa.lock` merges as a commutative monoid (`Table::fuse`). Multiple PRs touching strings merge cleanly without conflicts.
* **Autonomous PR Translations:** Automatically translates community PRs into all target languages using your preferred API key or completely offline dictionaries.

---

## 3. 3-Minute Maintainer Setup

### Step 1: Add Configuration
Run `glossa init` or create `glossa.json` in your repository root:

```json
{
  "source_locale": "en",
  "target_locales": ["es", "fr", "de", "ja", "zh", "pt"],
  "framework": "auto",
  "include": ["src/**/*.{tsx,jsx,vue,svelte,html,rs,py,go}"],
  "exclude": ["**/node_modules/**", "**/dist/**", "**/tests/**"],
  "catalogs": {
    "format": "json",
    "directory": "locales"
  }
}
```

### Step 2: Add GitHub Actions CI Workflow
Create `.github/workflows/i18n.yml`:

```yaml
name: Localization CI

on:
  push:
    branches: [main, master]
  pull_request:
    branches: [main, master]

jobs:
  check-i18n:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout Code
        uses: actions/checkout@v4

      - name: Install Glossa
        run: |
          curl -fsSL https://ptyktos.com/install.sh | bash
          sudo mv ~/.glossa/bin/glossa /usr/local/bin/

      - name: Verify String Coverage & Placeholder Safety
        run: |
          glossa extract
          glossa check --ci --format=github
```

If a contributor adds new UI text without updating translations, Glossa outputs rich GitHub annotations pointing directly to the missing lines and exits with code 1.

---

## 4. Contributor Guide Snippet (Add to `CONTRIBUTING.md`)

Add the following section to your project's `CONTRIBUTING.md`:

```markdown
## Internationalization (i18n)

This repository uses [Glossa](https://github.com/Ptyktos/glossa) for high-speed, multi-framework localization.

### Writing Code with Text
When adding or modifying user-facing text:
1. Wrap strings in the localization accessor:
   - **React:** `t('Welcome back')` or `<Trans>Welcome back</Trans>`
   - **Vue:** `{{ $t('Welcome back') }}`
   - **Svelte:** `{$t('Welcome back')}`
2. Keep dynamic parameters clear: `{userName}`, `{itemCount}`.
3. You do NOT need to manually translate text into other languages—our CI bot handles translations automatically!

### Checking Your Changes Locally
To verify localization before pushing:
```bash
# Install standalone binary (no Node.js required)
curl -fsSL https://ptyktos.com/install.sh | bash

# Check coverage
glossa extract && glossa check
```
```

---

## 5. Local Pre-Commit Hook Setup

To prevent broken translations from ever reaching Git, encourage contributors to enable a pre-commit hook.

### Using `.pre-commit-config.yaml`

```yaml
repos:
  - repo: local
    hooks:
      - id: glossa-check
        name: Glossa Localization Check
        entry: glossa check
        language: system
        pass_filenames: false
        always_run: true
```

### Or using a Native Git Hook (`.git/hooks/pre-commit`)

```bash
#!/bin/sh
# Glossa Pre-Commit Verification Hook
if command -v glossa >/dev/null 2>&1; then
    glossa extract --output /tmp/glossa_precommit.json >/dev/null 2>&1
    glossa check || {
        echo "❌ Glossa: Missing or invalid translations detected. Run 'glossa diff' to inspect."
        exit 1
    }
fi
exit 0
```

---

## 6. Zero-Budget AI Translation Strategies

For open-source projects with no corporate sponsor or API budget, Glossa supports three zero-cost translation options:

1. **Offline Translation Provider (`--provider offline`):** Uses deterministic dictionary mapping and pseudo-localization to test UI expansion without external network requests.
2. **Local Ollama Inference (`--provider ollama`):** Run Llama 3 or DeepSeek locally on maintainer hardware at zero cost.
3. **Free Community API Keys (DeepSeek / Groq):** Maintainers can add a free API token in GitHub Repository Secrets (`${{ secrets.GLOSSA_API_KEY }}`) to auto-translate PRs on merge.
