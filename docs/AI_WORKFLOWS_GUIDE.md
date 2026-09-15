# Integrating Glossa into Modern AI Workflows

> **Scope:** Cursor, Windsurf, Claude Code, GitHub Copilot, Model Context Protocol (MCP), and Autonomous Coding Agents.  
> **Objective:** Turn your AI coding assistants into zero-latency, localization-aware development partners that extract, verify, and translate UI text automatically.

---

## 1. Overview: The Problem with AI-Generated UI Copy

When developers use AI coding assistants (Cursor, Claude Code, Windsurf, Copilot) to generate frontend components, two recurring failures occur:
1. **Hardcoded Strings:** The AI spits out hardcoded English strings inside JSX, Vue, or Svelte templates instead of using localized message accessors.
2. **Hallucinated or Corrupted Interpolations:** When asked to internationalize code, LLMs often drop parameters (`{count}`), rename slots, corrupt HTML markup tags (`<0>...</0>`), or generate inconsistent dictionary keys.

**Glossa fixes this at the compiler level**:
* Provides an embedded **Model Context Protocol (MCP) server** (`glossa mcp`) that exposes real-time string extraction, lockfile search, and translation validation tools directly to your AI IDE.
* Runs a sub-microsecond **Integrity Gate** that mathematically guarantees slot and tag conservation before code is written to disk.
* Compiles **zero-runtime TypeScript and Rust accessors** (`glossa codegen`) so your AI assistant gets instant autocomplete on all localized keys.

---

## 2. Model Context Protocol (MCP) Integration

Glossa includes a built-in JSON-RPC 2.0 MCP server that runs over standard input/output (`stdio`).

### Configuring Glossa MCP in Cursor / Windsurf / Claude Code

Add the following to your MCP client configuration (e.g. `~/.cursor/mcp.json`, `claude_desktop_config.json`, or Windsurf settings):

```json
{
  "mcpServers": {
    "glossa": {
      "command": "glossa",
      "args": ["mcp"],
      "env": {
        "GLOSSA_DIR": "."
      }
    }
  }
}
```

### Exposed MCP Tools

Once enabled, your AI assistant gains access to four sovereign tools:

| Tool Name | Parameters | Description |
|---|---|---|
| `glossa_extract` | `files: string[]` | Extracts all translatable text from source code in microseconds, returning canonical AST terms and slots. |
| `glossa_validate` | `source: string`, `translation: string` | Runs the Integrity Gate to verify slot conservation and markup safety. |
| `glossa_lookup` | `query: string`, `locale: string` | Queries local Translation Memory (TM) using fuzzy matching to reuse existing approved translations. |
| `glossa_status` | `None` | Returns project translation coverage statistics, missing locales, and pending deltas. |

---

## 3. Configuring Cursor, Windsurf, and Claude Code

To instruct your AI agent to localize code automatically, add the following prompt rules to your repository.

### For Cursor (`.cursorrules` in repository root)

```markdown
# Localization Instructions (Glossa)

1. NEVER hardcode user-facing strings in UI components.
2. When creating or modifying React, Vue, or Svelte components:
   - Wrap user-facing copy in the project's localization accessor (e.g., `t('Button Label')` or `<Trans>Welcome</Trans>`).
   - Run `glossa extract` to register new strings into the canonical lockfile.
3. If providing translations:
   - Always preserve dynamic slots exactly as formatted: `{user}`, `{count}`, `<0>...</0>`.
   - Never remove or alter HTML tags within translatable strings.
4. Run `glossa check --ci` to verify coverage before finalizing your changes.
```

### For Claude Code (`CLAUDE.md` in repository root)

```markdown
# Localization Policy

- **Engine:** Glossa (Sovereign Rust Engine).
- **Commands:**
  - `glossa extract`: Scans codebase and updates translation coordinates.
  - `glossa translate --provider <provider>`: Translates missing strings with Integrity Gate verification.
  - `glossa check --ci`: Fails if translations are missing or placeholders are corrupted.
- **Rule:** Never allow hardcoded English text in components. Verify all translations with `glossa check` before submitting a PR.
```

---

## 4. Autonomous Agent Workflow (Multi-Agent Swarms)

When building autonomous coding agent swarms (e.g. using LangChain, AutoGPT, or custom agent frameworks), Glossa provides a deterministic, zero-token verification loop:

```
[Agent Generates UI Component]
              │
              ▼
    [glossa extract]  ──► (Extracts translatable strings in 7.7 µs)
              │
              ▼
   [glossa translate] ──► (Generates target translations)
              │
              ▼
   [Integrity Gate]   ──► PASS ──► Commit & Emit Catalog
              │
             FAIL
              │
              ▼
   [Microsecond Rejection] ──► Agent receives exact error (e.g., "Missing slot {count}")
                               and regenerates only the faulty key without burning tokens.
```

---

## 5. Automated PR Translation Bot (GitHub Actions)

You can deploy an automated localization bot on GitHub that automatically detects missing strings on every pull request, translates them using your AI provider of choice, and commits the updated lockfile back to the PR branch.

Create `.github/workflows/glossa-bot.yml`:

```yaml
name: Glossa Automated Translation Bot

on:
  pull_request:
    paths:
      - 'src/**'
      - 'locales/**'

jobs:
  localize:
    runs-on: ubuntu-latest
    permissions:
      contents: write
      pull-requests: write

    steps:
      - name: Checkout Code
        uses: actions/checkout@v4
        with:
          ref: ${{ github.head_ref }}

      - name: Install Glossa
        run: |
          curl -fsSL https://ptyktos.com/install.sh | bash
          sudo mv ~/.glossa/bin/glossa /usr/local/bin/

      - name: Extract & Verify Coverage
        run: |
          glossa extract
          glossa diff

      - name: Auto-Translate Missing Strings
        env:
          ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
        run: |
          glossa translate --provider anthropic --locales es,fr,de,ja
          glossa sync

      - name: Verify Integrity
        run: |
          glossa check --ci --format=github

      - name: Commit Updated Translations
        uses: stefanzweifel/git-auto-commit-action@v5
        with:
          commit_message: "chore(i18n): auto-translate missing strings via Glossa [skip ci]"
          file_pattern: "glossa.lock locales/**"
```

---

## 6. Zero-Runtime Type-Safe Code Generation

To eliminate runtime localization bugs entirely, use Glossa's compiler to generate fully typed message accessors for your AI tools:

```bash
glossa codegen --target typescript --output src/i18n/messages.ts
```

This generates zero-overhead TypeScript functions with typed slot arguments:

```typescript
// Auto-generated by Glossa - DO NOT EDIT MANUALLY
export const messages = {
  welcomeBanner: (slots: { user: string; count: number }) =>
    `Welcome ${slots.user}, you have ${slots.count} notifications.`,
  logoutButton: () => `Sign Out`,
} as const;
```

When your AI assistant writes frontend code, TypeScript's compiler enforces that `{ user, count }` are passed correctly, eliminating 100% of runtime parameter bugs before code is committed.
