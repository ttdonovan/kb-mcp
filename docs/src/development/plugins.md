---
description: "Read before installing kb-mcp's Claude Code plugins or changing them: the kb-mcp plugin for users and the kb-mcp-dev plugin for contributors"
---

# Claude Code Plugins

This repository is a Claude Code plugin marketplace with two plugins, both under `plugins/`:

- `kb-mcp`, for anyone using kb-mcp: a skill for using its tools and setting it up for a project.
- `kb-mcp-dev`, for people changing this repository: skills for its kinds of change, two read-only reviewers, and an MCP server built from the checkout. Kept apart so that a user never loads them.

Files: `.claude-plugin/marketplace.json` (the marketplace), `plugins/kb-mcp/` and `plugins/kb-mcp-dev/` (the plugins, each with `.claude-plugin/plugin.json`).

## The kb-mcp plugin

```sh
claude plugin marketplace add ttdonovan/kb-mcp
claude plugin install kb-mcp@kb-mcp
```

It adds one skill, `kb-mcp:kb-mcp`: how to search and read a knowledge base through the tools, and how to install kb-mcp, write `collections.ron` and register the server.

It adds no MCP server. kb-mcp serves the collections a project's `collections.ron` names, so the server belongs to the project: a plugin's server would start in every project, and fail in each one without a config. Register it in the project's `.mcp.json`, as in [MCP Server](../usage/mcp-server.md).

A plugin cannot reach files outside its own folder, so the skill links to this book and the repository by URL.

## The kb-mcp-dev plugin

| Skill | Use it to |
|---|---|
| `/kb-mcp-dev:add-tool` | add an MCP tool with its CLI command, tests and docs |
| `/kb-mcp-dev:review-code [branch]` | hand a change to code, scripts, CI or plugins to the read-only `code-reviewer` |
| `/kb-mcp-dev:review-docs [branch]` | hand a change to the docs to the read-only `docs-reviewer` |
| `/kb-mcp-dev:test-with-kb-dev` | test a change through the `kb-dev` server from the session: every tool, the error paths and automatic reindexing |
| `/kb-mcp-dev:work-in-a-stack` | split a change into a stack of pull requests with `gh stack`, and review each layer |
| `/kb-mcp-dev:release` | check, merge and confirm a release; only you can start it |
| `kb-mcp-dev:mdbook-docs` | general mdBook and Mermaid help; for either book, `docs/CLAUDE.md` or `book/CLAUDE.md` holds the project's rules and comes first |
| `kb-mcp-dev:obsidian-markdown`, `obsidian-bases`, `obsidian-cli` | work on the Obsidian vault in `vault/`; copied from upstream, see `skills/UPSTREAM.md` |

What the reviewers check, and their report, are in [Reviewing a Change](reviewing.md).

### The kb-dev MCP server

The plugin also adds an MCP server, `kb-dev`, that runs kb-mcp built from the checkout, so a session can try a change through a real MCP client. Its wrapper, `plugins/kb-mcp-dev/scripts/kb-mcp-debug.sh`:

- finds the checkout from the plugin's own folder when loaded in place (`just claude`), otherwise from the session's project directory, and fails with a message outside a kb-mcp checkout;
- runs `cargo build --locked -p kb-mcp-server`, which takes a few seconds when nothing changed and longer on a first build, then starts `target/debug/kb-mcp`;
- serves `collections.ron` at the root if you have one (it is gitignored), otherwise `collections.example.ron`, which indexes this repository; `KB_MCP_CONFIG` overrides it;
- keeps the index in `.ai/kb-dev/index/` (gitignored, beside the review logs), apart from an installed kb-mcp's, so `cargo clean` does not remove it; `KB_MCP_CACHE_DIR` overrides it.

After changing the code, reconnect `kb-dev` in `/mcp`: the wrapper rebuilds and restarts it. If a first build takes longer than the session waits for the server, reconnect once it finishes. In `/mcp` it is listed as `plugin:kb-mcp-dev:kb-dev`.

- `just claude` starts a session with it loaded from the checkout, so an edit applies on the next session or after `/reload-plugins` (arguments go to `claude`, such as `just claude --resume`). It is also how a session here gets kb-mcp's tools: the repository has no `.mcp.json`.
- To have it in every session here instead: `claude plugin marketplace add ./` from the root, then `claude plugin install kb-mcp-dev@kb-mcp --scope local`. `--scope project` would write it into the shared `.claude/settings.json`.
- `.claude/` holds only the committed `settings.json`, whose deny rules a plugin cannot ship, and each contributor's `settings.local.json`.
- A new skill goes in `plugins/kb-mcp-dev/skills/<name>/SKILL.md` and a new agent in `plugins/kb-mcp-dev/agents/<name>.md`, each with a `description` that says when to use it. Add a row to the table above and run `just plugin-validate`. Keep a skill to steps and pointers into this book or `CONTRIBUTING.md`, so a rule is written once.

## Versions

Neither `plugin.json` sets a `version`, so each commit to `main` is a new version of both: `claude plugin update` picks up a change with no release. `just plugin-validate` warns about the missing `version`, which is intended, and the missing icon, which only matters for a listing in Anthropic's plugin directory. The plugins are not tied to a kb-mcp release; the user skill describes the latest one.
