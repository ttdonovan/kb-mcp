---
name: add-tool
description: Add an MCP tool to kb-mcp with its CLI equivalent, shared logic in kb-core, tests and docs, so the two transports stay in parity. Use when asked to add a tool or a CLI command to kb-mcp, or to change an existing tool's parameters.
---

# Add a tool

The steps and a template are in `docs/src/development/adding-tools.md`; the rules they serve are "Hard Rules" in `CLAUDE.md`. This skill is the order and what is easy to miss.

## Steps

1. Put the logic in `crates/kb-core/src/`, with no rmcp, schemars or clap. Output goes through a `kb_core::format::*` function, so the tool and the CLI print the same JSON.
2. Add the tool: `crates/kb-mcp-server/src/tools/<name>.rs`, `pub mod <name>;` and `+ <name>::router()` in `tools/mod.rs`. Every parameter has a doc comment: it is the description an agent reads in the schema.
3. Add the CLI subcommand in `crates/kb-cli/src/main.rs`, with the same parameters and defaults.
4. Name the tool in the server instructions, `get_info` in `crates/kb-mcp-server/src/server.rs`.
5. Test the shared logic in kb-core.
6. Update the docs: `docs/src/reference/tools.md`, `docs/src/usage/cli.md`, the Tools table in `README.md`, and the tool count and tree in `CLAUDE.md`. If agents should use it, the user plugin's skill, `plugins/kb-mcp/skills/kb-mcp/SKILL.md`.
7. Run `just ci`, then try both transports: `cargo run -p kb-cli -- <command> ...`, and the tool from an MCP client.
8. Review with `/kb-mcp-dev:review-code`, and `/kb-mcp-dev:review-docs` for the docs.

## Gotchas

- A new tool or optional parameter is `feat:`. Renaming or removing one, or changing a default or the JSON it returns, is breaking: `feat!:` with what breaks in the body (`CONTRIBUTING.md`, Versioning).
- The server writes nothing to stdout but the protocol: log with `tracing`, which goes to stderr.
- A tool that reads documents reads them fresh from disk (`read_fresh`), not from the index.
- A tool that writes checks `writable` and that the path stays inside the collection, as `kb_write` does.
