# CLAUDE.md — kb-mcp

## What This Is

Standalone Rust binary crate — MCP server + CLI for markdown knowledge bases.
Indexes markdown collections (configured via RON) into a BM25 search index
and exposes 6 MCP tools via rmcp (stdio transport).

## Architecture

```
kb-mcp/
├── Cargo.toml
├── collections.ron              # Local dev config (gitignored)
├── collections.example.ron      # Template for users
├── src/
│   ├── main.rs                  # Dual-mode entry (MCP stdio / CLI)
│   ├── config.rs                # RON config loading + resolution chain
│   ├── index.rs                 # Document scanning, frontmatter parsing
│   ├── search.rs                # Tantivy BM25 search engine
│   ├── format.rs                # JSON output formatting
│   ├── types.rs                 # Core data types (Document, Section)
│   ├── cli.rs                   # Clap CLI subcommands
│   ├── server.rs                # rmcp ServerHandler
│   └── tools/
│       ├── mod.rs               # Router composition
│       ├── sections.rs          # list_sections
│       ├── search.rs            # search
│       ├── documents.rs         # get_document
│       ├── context.rs           # kb_context
│       ├── write.rs             # kb_write
│       └── reindex.rs           # reindex
├── book/                        # mdBook documentation
└── tests/
```

## Key Patterns

- **Dual-mode binary:** No args → MCP stdio server. With args → CLI.
- **RON config:** `collections.ron` defines all collections, sections, and descriptions.
  Zero hardcoded project-specific values in the binary.
- **Config resolution:** `--config` → `KB_MCP_CONFIG` env → `./collections.ron` → `~/.config/kb-mcp/collections.ron`
- **Collection paths** resolve relative to the config file's parent directory.
- **Tool pattern:** One file per tool in `src/tools/`. Each has params struct
  (`Deserialize + JsonSchema`), router function, and `#[rmcp::tool]` annotation.
  Routers composed with `+` in `tools/mod.rs`.
- **Logs to stderr** — stdout is the MCP JSON-RPC transport.

## Hard Rules

1. **No hardcoded paths or project-specific values.** Everything comes from `collections.ron`.
2. **All MCP tools must have CLI equivalents.** Parity between modes.
3. **get_document reads from disk** — the index is for search/lookup only. Fresh content always.
4. **kb_write only writes to collections with `writable: true`** — error with actionable message otherwise.

## Code Quality Rules

### Think Before Coding
- State assumptions explicitly. If uncertain, ask.
- If multiple approaches exist, present them — don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

### Simplicity First
- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.

### Surgical Changes
- Touch only what you must.
- Don't "improve" adjacent code, comments, or formatting.
- Match existing style. Every changed line should trace to the request.

### Rust Documentation
- Every doc comment explains **why**, not **what**. The code shows what — docs
  explain the design choice, tradeoff, or constraint that isn't obvious from reading it.
- Module-level `//!` docs on every file explaining the module's role and key design decisions.
- Doc comments (`///`) on public types and non-trivial functions.
- Don't doc the obvious — `/// Returns the path` on `fn path()` is noise.

### Goal-Driven Execution
Transform tasks into verifiable goals:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
```

## Common Commands

```sh
just              # List commands
just build        # cargo build
just release      # cargo build --release
just check        # cargo check
just clippy       # cargo clippy
just test         # cargo test
just install      # cargo install --path .
just book-build   # Build mdBook docs
just book-serve   # Serve docs with live reload
```

## Verification

After making changes, verify with:

- `cargo build` — compiles without errors
- `cargo clippy` — no new warnings
- `cargo test` — tests pass
- `just book-build` — mdBook builds cleanly (if docs changed)
- `cargo run -- list-sections` — CLI works against `collections.ron`
- `cargo run -- search --query "test"` — search returns results

## Development Tooling

This project was built using [Claude Code](https://claude.com/claude-code)
with the [Compound Engineering](https://every.to/guides/compound-engineering)
plugin — an AI-native dev methodology where each work cycle makes future
cycles easier. The Plan → Work → Review → Compound loop drove the
brainstorm, plan, and implementation phases.

Key tools used:
- `/ce:brainstorm` — explored requirements and design decisions
- `/ce:plan` — transformed the brainstorm into an implementation plan
- `/ce:work` — executed the plan with task tracking
- Parallel research agents — repo analysis, external crate docs, spec flow analysis

The brainstorm and plan are preserved in `docs/brainstorms/` and `docs/plans/`.

## Adding a New Tool

1. Create `src/tools/my_tool.rs` with params struct and `#[rmcp::tool]` impl
2. Add `pub(crate) mod my_tool;` to `src/tools/mod.rs`
3. Add `+ my_tool::router()` to `combined_router()`
4. Add corresponding CLI subcommand in `src/cli.rs`
5. Update server instructions in `src/server.rs`
