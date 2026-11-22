# Using Glossa with AI coding assistants

Glossa includes a Model Context Protocol (MCP) server and a command-line workflow that fit into
AI-assisted development: an assistant can extract strings, check translations and read coverage.

## MCP server

`glossa mcp` starts a JSON-RPC 2.0 server over standard input/output.

Add it to your MCP client configuration (for example `~/.cursor/mcp.json` or
`claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "glossa": {
      "command": "glossa",
      "args": ["mcp"]
    }
  }
}
```

### Tools

| Tool | Arguments | What it does |
|---|---|---|
| `extract_strings` | `code`, `format` (`jsx`, `vue`, `svelte`, `html`, `md`, `json`), optional `file_path` | Extracts translatable strings and their slots from a snippet. |
| `translate_string` | `text`, `target_locale` | Translates one string and runs the integrity gate on the result. The server uses the built-in `offline` provider, a rule-based development provider, not a real translation model. |
| `compile_component` | `key`, `text`, `target_framework` (`react`, `vue`, `svelte`, `solid`) | Returns the framework call (for example a `t()` or `<Trans>` call) for a string. |
| `get_status` | optional `dir` | Returns the number of lockfile entries, target locales, pending translations, the configured provider and the lockfile version. |

## Assistant rules

Add rules to the instructions file your assistant reads (`.cursorrules`, `CLAUDE.md`, and so on).
For example:

```markdown
# Localization

- Do not hardcode user-facing strings in UI components; use the project's localization accessor.
- Run `glossa extract` after adding strings, and `glossa check --ci` before finishing.
- When writing translations, keep slots (`{user}`, `{count}`) and numbered tags (`<0>...</0>`)
  exactly as in the source text.
```

## Command-line workflow

```
glossa extract                         # scan sources, update the lockfile
glossa translate --provider <name>     # translate missing entries; each result goes through the integrity gate
glossa sync                            # write locale catalogs from the lockfile
glossa check --ci --format=github      # exit code 1 if any configured locale is missing a translation
```

The integrity gate rejects a translation that drops or adds a `{slot}`, unbalances or reorders
numbered tags, contains script or event-handler markup, or changes a protected glossary term. A
rejected translation is reported with the reason, so it can be regenerated.

## Pull request automation

A workflow that runs the commands above on pull requests, with `ANTHROPIC_API_KEY` or
`OPENAI_API_KEY` supplied as a secret, can translate missing strings and commit the updated
lockfile and catalogs. Install the binary first (see the README), then run `glossa extract`,
`glossa translate --provider anthropic --locales es,fr,de`, `glossa sync` and `glossa check --ci`.

## Generated accessors

```bash
glossa codegen --target typescript --output src/i18n/messages.ts
```

generates TypeScript (or, with `--target rust`, Rust) accessors from the lockfile. Entries with
slots take a typed parameter object, so the compiler checks that the slots are passed:

```typescript
// shape of generated code
welcomeBanner: (params: { user: string | number; count: string | number }, locale: Locale = defaultLocale): string => { /* ... */ }
```
